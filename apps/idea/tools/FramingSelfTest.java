package dev.gitnova.idea.transport;

import java.io.ByteArrayInputStream;
import java.io.IOException;
import java.nio.charset.StandardCharsets;

public final class FramingSelfTest {
    public static void main(String[] args) throws Exception {
        String json = "{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":\"安全\"}";
        if (!Framing.read(new ByteArrayInputStream(Framing.frame(json))).equals(json)) throw new AssertionError("round trip");
        expectFailure("Content-Length: 1\r\nContent-Length: 1\r\n\r\nx");
        expectFailure("Content-Length: 4\r\n\r\n{}");
        expectFailure("Content-Length: " + (Framing.MAX_FRAME_BYTES + 1) + "\r\n\r\n");
        System.out.println("JetBrains framing tests passed");
    }

    private static void expectFailure(String value) throws Exception {
        try {
            Framing.read(new ByteArrayInputStream(value.getBytes(StandardCharsets.UTF_8)));
            throw new AssertionError("expected framing failure");
        } catch (IOException expected) {
            // Expected.
        }
    }
}
