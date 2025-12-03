// Provide a global WebSocket for Node environments.
import WebSocket from "ws";

const g = globalThis as Record<string, unknown>;
if (!g.WebSocket) {
  g.WebSocket = WebSocket;
}
