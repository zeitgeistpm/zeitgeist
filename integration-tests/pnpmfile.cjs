/**
 * Force polkadot-api ws-provider imports to use the Node build instead of the web build.
 *
 * Moonwall pulls in `polkadot-api/ws-provider/web`, which expects a global WebSocket.
 * We rewrite the "web" entry points to the Node build so CI can run under Node 20
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

const deepSwapToNode = (value) => {
  if (typeof value === "string") {
    return value
      .replace(/ws-provider_web/g, "ws-provider_node")
      .replace(/ws-provider(\b|$)/g, "ws-provider_node");
  }
  if (value && typeof value === "object") {
    return Object.fromEntries(Object.entries(value).map(([k, v]) => [k, deepSwapToNode(v)]));
  }
  return value;
};

module.exports = {
  hooks: {
    readPackage(pkg) {
      // polkadot-api subpath packages that pnpm generates internally
      if (pkg.name === "polkadot-api_ws-provider_web") {
        redirectToNode(pkg);
      }
      if (pkg.name === "polkadot-api_ws-provider") {
        redirectToNode(pkg);
      }

      // direct package: keep web subpath aligned with node build
      if (pkg.name === "@polkadot-api/ws-provider") {
        const nodeExport = pkg.exports?.["./node"];
        if (nodeExport) {
          pkg.exports["./web"] = nodeExport;
        }
      }

      // top-level reexports: point web entry at the node build as well
      if (pkg.name === "polkadot-api") {
        if (pkg.exports?.["./ws-provider"]) {
          pkg.exports["./ws-provider/web"] = deepSwapToNode(pkg.exports["./ws-provider"]);
        }
      }

      return pkg;
    },
  },
};
