import { spawn } from "node:child_process";
import { randomBytes } from "node:crypto";

const bridgeToken = randomBytes(32).toString("hex");
const allowedOrigin = "http://127.0.0.1:4174";

const bridge = spawn(process.execPath, ["scripts/bridge-server.mjs"], {
  stdio: "inherit",
  env: {
    ...process.env,
    BRIDGE_PORT: process.env.BRIDGE_PORT || "8787",
    BRIDGE_TOKEN: bridgeToken,
    ALLOWED_ORIGIN: allowedOrigin,
  },
});

const vite = spawn("npm", ["run", "dev", "--", "--host", "127.0.0.1", "--port", "4174"], {
  stdio: "inherit",
  env: {
    ...process.env,
    VITE_EXECUTION_MODE: "bridge",
    VITE_EXECUTION_BRIDGE_URL: process.env.VITE_EXECUTION_BRIDGE_URL || "http://127.0.0.1:8787/execute",
    VITE_EXECUTION_BRIDGE_TOKEN: bridgeToken,
  },
});

const cleanup = () => {
  bridge.kill("SIGTERM");
  vite.kill("SIGTERM");
};

process.on("SIGINT", cleanup);
process.on("SIGTERM", cleanup);

bridge.on("exit", (code) => {
  if (code && code !== 0) {
    process.exit(code);
  }
});

vite.on("exit", (code) => {
  cleanup();
  process.exit(code ?? 0);
});
