package dev.gitnova.idea.transport;

import java.io.ByteArrayOutputStream;
import java.io.EOFException;
import java.io.IOException;
import java.io.InputStream;
import java.nio.charset.StandardCharsets;

public final class Framing {
    public static final int MAX_FRAME_BYTES = 16 * 1024 * 1024;

    private Framing() {}

    public static byte[] frame(String json) {
        byte[] body = json.getBytes(StandardCharsets.UTF_8);
        if (body.length > MAX_FRAME_BYTES) throw new IllegalArgumentException("Core request is too large");
        byte[] header = ("Content-Length: " + body.length + "\r\n\r\n").getBytes(StandardCharsets.US_ASCII);
        byte[] framed = new byte[header.length + body.length];
        System.arraycopy(header, 0, framed, 0, header.length);
        System.arraycopy(body, 0, framed, header.length, body.length);
        return framed;
    }

    public static String read(InputStream input) throws IOException {
        ByteArrayOutputStream header = new ByteArrayOutputStream();
        int matched = 0;
        while (matched < 4) {
            int value = input.read();
            if (value < 0) throw new EOFException("Core stdout closed");
            header.write(value);
            byte expected = new byte[] {'\r', '\n', '\r', '\n'}[matched];
            matched = value == expected ? matched + 1 : value == '\r' ? 1 : 0;
            if (header.size() > 8192) throw new IOException("Core frame header is too large");
        }
        String[] lines = header.toString(StandardCharsets.US_ASCII).split("\\r\\n");
        Integer length = null;
        for (String line : lines) {
            if (line.regionMatches(true, 0, "Content-Length:", 0, 15)) {
                if (length != null) throw new IOException("Duplicate Content-Length");
                try { length = Integer.parseInt(line.substring(15).trim()); }
                catch (NumberFormatException error) { throw new IOException("Invalid Content-Length", error); }
            }
        }
        if (length == null || length < 0 || length > MAX_FRAME_BYTES) throw new IOException("Invalid Content-Length");
        byte[] body = input.readNBytes(length);
        if (body.length != length) throw new EOFException("Incomplete Core frame");
        return new String(body, StandardCharsets.UTF_8);
    }
}
