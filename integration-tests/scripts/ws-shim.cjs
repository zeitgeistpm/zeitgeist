// Ensure a global WebSocket exists when running under Node.
if (typeof global.WebSocket === "undefined") {
  global.WebSocket = require("ws");
}
