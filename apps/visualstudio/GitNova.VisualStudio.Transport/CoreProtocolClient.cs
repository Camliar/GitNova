using System.Collections.Concurrent;
using System.Diagnostics;
using System.Text.Json;

namespace GitNova.VisualStudio.Transport;

public sealed class CoreProtocolClient : IAsyncDisposable
{
    private static readonly TimeSpan RequestTimeout = TimeSpan.FromSeconds(15);
    private readonly Process process;
    private readonly ConcurrentDictionary<long, TaskCompletionSource<JsonElement>> pending = new();
    private readonly SemaphoreSlim writer = new(1, 1);
    private readonly CancellationTokenSource lifetime = new();
    private long nextId;

    private CoreProtocolClient(Process process)
    {
        this.process = process;
        _ = ReadLoopAsync();
    }

    public static CoreProtocolClient Start(string? overridePath = null)
    {
        string executable = ResolveExecutable(overridePath);
        var start = new ProcessStartInfo
        {
            FileName = executable,
            UseShellExecute = false,
            RedirectStandardInput = true,
            RedirectStandardOutput = true,
            RedirectStandardError = true,
            CreateNoWindow = true,
        };
        Process process = Process.Start(start) ?? throw new InvalidOperationException("Unable to start gitnova-core.");
        _ = process.StandardError.ReadToEndAsync();
        return new CoreProtocolClient(process);
    }

    public static string ResolveExecutable(string? overridePath)
    {
        if (!string.IsNullOrWhiteSpace(overridePath))
        {
            if (!Path.IsPathFullyQualified(overridePath)) throw new ArgumentException("GITNOVA_CORE_PATH must be absolute.", nameof(overridePath));
            return overridePath;
        }
        return OperatingSystem.IsWindows() ? "gitnova-core.exe" : "gitnova-core";
    }

    public async Task<JsonElement> RequestAsync(string method, object? parameters, CancellationToken cancellationToken)
    {
        long id = Interlocked.Increment(ref nextId);
        var completion = new TaskCompletionSource<JsonElement>(TaskCreationOptions.RunContinuationsAsynchronously);
        if (!pending.TryAdd(id, completion)) throw new InvalidOperationException("Duplicate Core request id.");
        string request = JsonSerializer.Serialize(new { jsonrpc = "2.0", id, method, @params = parameters });
        await writer.WaitAsync(cancellationToken);
        try { await ContentLengthFraming.WriteAsync(process.StandardInput.BaseStream, request, cancellationToken); }
        finally { writer.Release(); }
        using var timeout = CancellationTokenSource.CreateLinkedTokenSource(cancellationToken, lifetime.Token);
        timeout.CancelAfter(RequestTimeout);
        using CancellationTokenRegistration registration = timeout.Token.Register(() => completion.TrySetCanceled(timeout.Token));
        try { return await completion.Task; }
        finally { pending.TryRemove(id, out _); }
    }

    public async Task NotifyAsync(string method, object? parameters, CancellationToken cancellationToken)
    {
        string notification = JsonSerializer.Serialize(new { jsonrpc = "2.0", method, @params = parameters });
        await writer.WaitAsync(cancellationToken);
        try { await ContentLengthFraming.WriteAsync(process.StandardInput.BaseStream, notification, cancellationToken); }
        finally { writer.Release(); }
    }

    private async Task ReadLoopAsync()
    {
        try
        {
            while (!lifetime.IsCancellationRequested)
            {
                string json = await ContentLengthFraming.ReadAsync(process.StandardOutput.BaseStream, lifetime.Token);
                using JsonDocument document = JsonDocument.Parse(json);
                if (!document.RootElement.TryGetProperty("id", out JsonElement idElement) || !idElement.TryGetInt64(out long id)) continue;
                if (!pending.TryRemove(id, out TaskCompletionSource<JsonElement>? completion)) continue;
                if (document.RootElement.TryGetProperty("error", out JsonElement error))
                {
                    string message = error.TryGetProperty("message", out JsonElement value) ? value.GetString() ?? "Core request failed." : "Core request failed.";
                    completion.TrySetException(new InvalidOperationException(message));
                }
                else if (document.RootElement.TryGetProperty("result", out JsonElement result)) completion.TrySetResult(result.Clone());
                else completion.TrySetException(new InvalidDataException("Core response has no result."));
            }
        }
        catch (OperationCanceledException) when (lifetime.IsCancellationRequested) { }
        catch (Exception error)
        {
            foreach (TaskCompletionSource<JsonElement> completion in pending.Values) completion.TrySetException(new IOException("Core transport failed.", error));
            pending.Clear();
        }
    }

    public async ValueTask DisposeAsync()
    {
        if (!process.HasExited)
        {
            using var shutdown = new CancellationTokenSource(TimeSpan.FromSeconds(2));
            try
            {
                await RequestAsync("gitnova/shutdown", null, shutdown.Token);
                await NotifyAsync("exit", null, shutdown.Token);
                await process.WaitForExitAsync(shutdown.Token);
            }
            catch (Exception) when (!process.HasExited) { process.Kill(entireProcessTree: true); }
        }
        lifetime.Cancel();
        process.Dispose();
        lifetime.Dispose();
        writer.Dispose();
    }
}
