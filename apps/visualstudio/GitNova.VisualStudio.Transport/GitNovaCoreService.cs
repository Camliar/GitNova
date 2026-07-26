using System.Text;
using System.Text.Json;

namespace GitNova.VisualStudio.Transport;

public sealed class GitNovaCoreService : IAsyncDisposable
{
    public const string ProtocolVersion = "1.15";
    private readonly SemaphoreSlim startup = new(1, 1);
    private CoreProtocolClient? client;

    public async Task<JsonElement> RequestAsync(string method, object? parameters, CancellationToken cancellationToken)
    {
        await EnsureStartedAsync(cancellationToken);
        return await client!.RequestAsync(method, parameters, cancellationToken);
    }

    public async Task<SquashTraceResult> LoadTraceAsync(string repositoryPath, int pullRequestNumber, string? selectedOid, CancellationToken cancellationToken)
    {
        if (!Path.IsPathFullyQualified(repositoryPath)) throw new ArgumentException("Repository path must be absolute.", nameof(repositoryPath));
        await RequestAsync("repository/open", new { path = repositoryPath }, cancellationToken);
        JsonElement repository = await RequestAsync("github/repository", new { }, cancellationToken);
        string nameWithOwner = repository.GetProperty("nameWithOwner").GetString() ?? throw new InvalidDataException("Core repository identity is missing.");
        JsonElement trace = await RequestAsync("github/squashTrace", new { number = pullRequestNumber, nameWithOwner }, cancellationToken);
        JsonElement? diff = null;
        if (!string.IsNullOrWhiteSpace(selectedOid))
            diff = await RequestAsync("github/pullRequestCommitDiff", new { number = pullRequestNumber, nameWithOwner, oid = selectedOid }, cancellationToken);
        return new SquashTraceResult(trace, diff);
    }

    private async Task EnsureStartedAsync(CancellationToken cancellationToken)
    {
        if (client is not null) return;
        await startup.WaitAsync(cancellationToken);
        try
        {
            if (client is not null) return;
            CoreProtocolClient candidate = CoreProtocolClient.Start(Environment.GetEnvironmentVariable("GITNOVA_CORE_PATH"));
            try
            {
                JsonElement initialized = await candidate.RequestAsync("gitnova/initialize", new
                {
                    clientInfo = new { name = "gitnova-visualstudio", version = "0.1.0" },
                    protocolVersion = ProtocolVersion,
                    capabilities = new { cancellation = true },
                }, cancellationToken);
                string remoteVersion = initialized.GetProperty("protocolVersion").GetString() ?? "";
                if (remoteVersion.Split('.')[0] != ProtocolVersion.Split('.')[0]) throw new InvalidOperationException("Core protocol is incompatible.");
                client = candidate;
            }
            catch { await candidate.DisposeAsync(); throw; }
        }
        finally { startup.Release(); }
    }

    public async ValueTask DisposeAsync()
    {
        if (client is not null) await client.DisposeAsync();
        startup.Dispose();
    }
}

public sealed record SquashTraceResult(JsonElement Trace, JsonElement? Diff)
{
    public string Render()
    {
        var output = new StringBuilder();
        JsonElement pullRequest = Trace.GetProperty("pullRequest");
        JsonElement relationship = Trace.GetProperty("relationship");
        output.AppendLine($"PR #{pullRequest.GetProperty("number")} — {pullRequest.GetProperty("title").GetString()}");
        output.AppendLine($"Relationship: {relationship.GetProperty("classification").GetString()} (confidence {relationship.GetProperty("confidence")})");
        output.AppendLine($"Final squash commit: {GetOptionalString(relationship, "mergeCommitOid")}");
        output.AppendLine("Original commits:");
        foreach (JsonElement commit in pullRequest.GetProperty("commits").EnumerateArray())
            output.AppendLine($"  {commit.GetProperty("oid").GetString()}  {commit.GetProperty("summary").GetString()}");
        if (Diff is JsonElement diff)
        {
            output.AppendLine("Selected original commit diff:");
            foreach (JsonElement file in diff.GetProperty("files").EnumerateArray())
            {
                output.AppendLine($"--- {GetOptionalString(file, "oldPath")}");
                output.AppendLine($"+++ {GetOptionalString(file, "newPath")}  +{file.GetProperty("additions")} -{file.GetProperty("deletions")}");
                foreach (JsonElement hunk in file.GetProperty("hunks").EnumerateArray())
                {
                    output.AppendLine(hunk.GetProperty("header").GetString());
                    foreach (JsonElement line in hunk.GetProperty("lines").EnumerateArray())
                    {
                        string prefix = line.GetProperty("kind").GetString() switch { "addition" => "+", "deletion" => "-", _ => " " };
                        output.AppendLine(prefix + line.GetProperty("content").GetString());
                    }
                }
            }
        }
        return output.ToString();
    }

    private static string GetOptionalString(JsonElement value, string property) =>
        value.TryGetProperty(property, out JsonElement item) && item.ValueKind == JsonValueKind.String ? item.GetString() ?? "unavailable" : "unavailable";
}
