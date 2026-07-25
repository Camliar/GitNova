using System.Text;
using System.Text.Json;
using GitNova.VisualStudio.Transport;

await RoundTripsUtf8Async();
await RejectsMissingLengthAsync();
RejectsRelativeExecutable();
RendersTraceAndDiff();
Console.WriteLine("Visual Studio transport tests passed");

static async Task RoundTripsUtf8Async()
{
    await using var stream = new MemoryStream();
    const string value = "{\"title\":\"GitNova 差异\"}";
    await ContentLengthFraming.WriteAsync(stream, value, CancellationToken.None);
    stream.Position = 0;
    string actual = await ContentLengthFraming.ReadAsync(stream, CancellationToken.None);
    Equal(value, actual, "UTF-8 framing round trip");
}

static async Task RejectsMissingLengthAsync()
{
    await using var stream = new MemoryStream(Encoding.ASCII.GetBytes("Other: 1\r\n\r\nx"));
    try { await ContentLengthFraming.ReadAsync(stream, CancellationToken.None); }
    catch (InvalidDataException) { return; }
    throw new Exception("Missing Content-Length was accepted.");
}

static void RejectsRelativeExecutable()
{
    try { CoreProtocolClient.ResolveExecutable("tools/gitnova-core"); }
    catch (ArgumentException) { return; }
    throw new Exception("Relative Core override was accepted.");
}

static void RendersTraceAndDiff()
{
    using JsonDocument trace = JsonDocument.Parse("""{"pullRequest":{"number":7,"title":"Ship trace","commits":[{"oid":"abc123","summary":"first"}]},"relationship":{"classification":"exact","confidence":1.0,"mergeCommitOid":"def456"}}""");
    using JsonDocument diff = JsonDocument.Parse("""{"files":[{"oldPath":"a.txt","newPath":"a.txt","additions":1,"deletions":0,"hunks":[{"header":"@@ -0,0 +1 @@","lines":[{"kind":"addition","content":"hello"}]}]}]}""");
    string output = new SquashTraceResult(trace.RootElement.Clone(), diff.RootElement.Clone()).Render();
    if (!output.Contains("abc123", StringComparison.Ordinal) || !output.Contains("+hello", StringComparison.Ordinal)) throw new Exception("Trace render omitted commit or line diff.");
}

static void Equal(string expected, string actual, string name)
{
    if (!string.Equals(expected, actual, StringComparison.Ordinal)) throw new Exception($"{name} failed.");
}
