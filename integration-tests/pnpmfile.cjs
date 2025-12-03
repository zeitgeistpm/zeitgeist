/**
 * Force polkadot-api ws-provider imports to use the Node build instead of the web build.
 *
 * Moonwall pulls in `polkadot-api/ws-provider/web`, which expects a global WebSocket.
 * Here we rewrite the web reexports to point at the node entry so CI can run under Node 20
 * without shimming globals.
 */
const redirectToNode = (pkg) => {
  const swap = (value) =>
    typeof value === "string" ? value.replace("ws-provider_web", "ws-provider_node") : value;
  pkg.module = swap(pkg.module);
  pkg.import = swap(pkg.import);
  pkg.browser = swap(pkg.browser);
  pkg.require = swap(pkg.require);
  pkg.default = swap(pkg.default);
};

module.exports = {
  hooks: {
    readPackage(pkg) {
      // polkadot-api reexports used by Moonwall
      if (pkg.name === "polkadot-api_ws-provider_web") {
        redirectToNode(pkg);
      }
      if (pkg.name === "polkadot-api_ws-provider") {
        redirectToNode(pkg);
      }
      // direct package: keep web subpath aligned with node build
      if (pkg.name === "@polkadot-api/ws-provider" && pkg.exports?.["./web"]) {
        pkg.exports["./web"] = pkg.exports["./node"];
      }
      return pkg;
    },
  },
};
