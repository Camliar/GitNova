package dev.gitnova.idea;

import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import com.intellij.openapi.Disposable;
import com.intellij.openapi.components.Service;
import dev.gitnova.idea.transport.CoreProtocolClient;
import java.nio.file.Path;

@Service(Service.Level.PROJECT)
public final class CoreService implements Disposable {
    private static final String PROTOCOL_VERSION = "1.15";
    private CoreProtocolClient client;

    public synchronized JsonObject request(String method, JsonObject params) throws Exception {
        ensureStarted();
        JsonObject envelope = JsonParser.parseString(client.request(method, params.toString())).getAsJsonObject();
        if (envelope.has("error")) {
            String message = envelope.getAsJsonObject("error").get("message").getAsString();
            throw new IllegalStateException(message);
        }
        if (!envelope.has("result")) throw new IllegalStateException("Core returned an invalid response");
        return envelope.get("result").isJsonObject() ? envelope.getAsJsonObject("result") : new JsonObject();
    }

    private void ensureStarted() throws Exception {
        if (client != null) return;
        String override = System.getProperty("gitnova.core.path", "").trim();
        if (!override.isEmpty() && !Path.of(override).isAbsolute()) throw new IllegalArgumentException("gitnova.core.path must be absolute");
        String executable = !override.isEmpty() ? override : System.getProperty("os.name", "").startsWith("Windows") ? "gitnova-core.exe" : "gitnova-core";
        CoreProtocolClient candidate = new CoreProtocolClient(executable);
        JsonObject capabilities = new JsonObject();
        capabilities.addProperty("cancellation", true);
        JsonObject clientInfo = new JsonObject();
        clientInfo.addProperty("name", "gitnova-idea");
        clientInfo.addProperty("version", "0.1.0");
        JsonObject initialize = new JsonObject();
        initialize.add("clientInfo", clientInfo);
        initialize.addProperty("protocolVersion", PROTOCOL_VERSION);
        initialize.add("capabilities", capabilities);
        JsonObject envelope = JsonParser.parseString(candidate.request("gitnova/initialize", initialize.toString())).getAsJsonObject();
        if (envelope.has("error") || !envelope.has("result")) {
            candidate.close();
            throw new IllegalStateException("Core initialization failed");
        }
        String version = envelope.getAsJsonObject("result").get("protocolVersion").getAsString();
        if (!version.split("\\.")[0].equals(PROTOCOL_VERSION.split("\\.")[0])) {
            candidate.close();
            throw new IllegalStateException("Core protocol is incompatible");
        }
        client = candidate;
    }

    @Override
    public synchronized void dispose() {
        if (client != null) client.close();
        client = null;
    }
}
