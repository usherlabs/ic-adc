import path from "node:path";
import { defineConfig } from "vite";
import dfxJson from "./example_caller/dfx.json";

// List of all aliases for canisters
const aliases = Object.entries(dfxJson.canisters).reduce((acc, [name, _value]) => {
  // Get the network name, or `local` by default.
  const networkName = process.env["DFX_NETWORK"] || "local";
  const outputRoot = path.join(__dirname, ".dfx", networkName, "canisters", name);

  return {
    ...acc,
    [`dfx-generated/${name}`]: path.join(outputRoot),
  };
}, {});

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [],
  resolve: {
    alias: {
      ...aliases,
    },
  },
  test: {
    globals: true,
    include: ["example_caller/__tests__/*.test.ts"], // Ensure your test path is included
  },
});
