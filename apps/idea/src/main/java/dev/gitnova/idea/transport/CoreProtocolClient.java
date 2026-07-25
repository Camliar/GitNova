package dev.gitnova.idea.transport;

import java.io.BufferedInputStream;
import java.io.BufferedOutputStream;
import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.time.Duration;
import java.util.Map;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicLong;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

public final class CoreProtocolClient implements AutoCloseable {
    private static final Pattern RESPONSE_ID = Pattern.compile("\\\"id\\\"\\s*:\\s*(\\d+)");
    private static final Duration TIMEOUT = Duration.ofSeconds(15);
    private final Process process;
    private final BufferedOutputStream output;
    private final AtomicLong nextId = new AtomicLong(1);
    private final Map<Long, CompletableFuture<String>> pending = new ConcurrentHashMap<>();

    public CoreProtocolClient(String executable) throws IOException {
        ProcessBuilder builder = new ProcessBuilder(executable);
        builder.redirectError(ProcessBuilder.Redirect.DISCARD);
        process = builder.start();
        output = new BufferedOutputStream(process.getOutputStream());
        Thread reader = new Thread(this::readLoop, "gitnova-core-stdout");
        reader.setDaemon(true);
        reader.start();
    }

    public String request(String method, String paramsJson) throws Exception {
        long id = nextId.getAndIncrement();
        CompletableFuture<String> response = new CompletableFuture<>();
        pending.put(id, response);
        String request = "{\"jsonrpc\":\"2.0\",\"id\":" + id + ",\"method\":\"" + method + "\",\"params\":" + paramsJson + "}";
        synchronized (output) {
            output.write(Framing.frame(request));
            output.flush();
        }
        try { return response.get(TIMEOUT.toMillis(), TimeUnit.MILLISECONDS); }
        finally { pending.remove(id); }
    }

    private void readLoop() {
        try (BufferedInputStream input = new BufferedInputStream(process.getInputStream())) {
            while (process.isAlive()) {
                String response = Framing.read(input);
                Matcher matcher = RESPONSE_ID.matcher(response);
                if (!matcher.find()) throw new IOException("Core response id is missing");
                CompletableFuture<String> future = pending.remove(Long.parseLong(matcher.group(1)));
                if (future != null) future.complete(response);
            }
        } catch (Exception error) {
            for (CompletableFuture<String> future : pending.values()) future.completeExceptionally(new IOException("Core transport failed"));
            pending.clear();
            process.destroyForcibly();
        }
    }

    public void notify(String method, String paramsJson) throws IOException {
        String message = "{\"jsonrpc\":\"2.0\",\"method\":\"" + method + "\",\"params\":" + paramsJson + "}";
        synchronized (output) {
            output.write(Framing.frame(message));
            output.flush();
        }
    }

    @Override
    public void close() {
        try {
            request("gitnova/shutdown", "null");
            notify("exit", "null");
            if (!process.waitFor(2, TimeUnit.SECONDS)) process.destroyForcibly();
        } catch (Exception error) {
            process.destroyForcibly();
        }
    }
}
