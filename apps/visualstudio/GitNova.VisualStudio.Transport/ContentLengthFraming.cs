using System.Globalization;
using System.Text;

namespace GitNova.VisualStudio.Transport;

public static class ContentLengthFraming
{
    private const int MaxHeaderBytes = 8 * 1024;
    private const int MaxContentBytes = 16 * 1024 * 1024;

    public static async Task WriteAsync(Stream output, string json, CancellationToken cancellationToken)
    {
        byte[] content = Encoding.UTF8.GetBytes(json);
        if (content.Length > MaxContentBytes) throw new InvalidDataException("Core message is too large.");
        byte[] header = Encoding.ASCII.GetBytes($"Content-Length: {content.Length}\r\n\r\n");
        await output.WriteAsync(header, cancellationToken);
        await output.WriteAsync(content, cancellationToken);
        await output.FlushAsync(cancellationToken);
    }

    public static async Task<string> ReadAsync(Stream input, CancellationToken cancellationToken)
    {
        var header = new List<byte>();
        while (!EndsWithHeaderTerminator(header))
        {
            if (header.Count >= MaxHeaderBytes) throw new InvalidDataException("Core header is too large.");
            int value = await ReadByteAsync(input, cancellationToken);
            if (value < 0) throw new EndOfStreamException("Core stdout closed.");
            header.Add((byte)value);
        }

        string text = Encoding.ASCII.GetString(header.ToArray());
        int length = ParseContentLength(text);
        byte[] content = new byte[length];
        await input.ReadExactlyAsync(content, cancellationToken);
        return Encoding.UTF8.GetString(content);
    }

    private static bool EndsWithHeaderTerminator(List<byte> bytes) =>
        bytes.Count >= 4 && bytes[^4] == '\r' && bytes[^3] == '\n' && bytes[^2] == '\r' && bytes[^1] == '\n';

    private static int ParseContentLength(string header)
    {
        foreach (string line in header.Split("\r\n", StringSplitOptions.RemoveEmptyEntries))
        {
            int separator = line.IndexOf(':');
            if (separator < 0 || !line[..separator].Equals("Content-Length", StringComparison.OrdinalIgnoreCase)) continue;
            if (!int.TryParse(line[(separator + 1)..].Trim(), NumberStyles.None, CultureInfo.InvariantCulture, out int length) || length < 0 || length > MaxContentBytes)
                throw new InvalidDataException("Core Content-Length is invalid.");
            return length;
        }
        throw new InvalidDataException("Core Content-Length is missing.");
    }

    private static async Task<int> ReadByteAsync(Stream stream, CancellationToken cancellationToken)
    {
        byte[] value = new byte[1];
        int count = await stream.ReadAsync(value, cancellationToken);
        return count == 0 ? -1 : value[0];
    }
}
