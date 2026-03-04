import { createServer } from "node:http";
import { existsSync } from "node:fs";
import { spawn, spawnSync } from "node:child_process";
import { resolve } from "node:path";

const port = Number(process.env.BRIDGE_PORT || 8787);
const workspaceRoot = resolve(process.cwd(), "..");
const manifestPath = resolve(workspaceRoot, "src-tauri", "Cargo.toml");
const binaryPath = resolve(workspaceRoot, "target", "debug", "gh-client-envelope-cli");

ensureBinary();

const server = createServer(async (req, res) => {
  const corsHeaders = {
    "access-control-allow-origin": "*",
    "access-control-allow-methods": "POST, GET, OPTIONS",
    "access-control-allow-headers": "content-type",
  };

  if (!req.url) {
    res.writeHead(400, corsHeaders).end();
    return;
  }

  if (req.method === "OPTIONS") {
    res.writeHead(204, corsHeaders).end();
    return;
  }

  if (req.method === "GET" && req.url === "/health") {
    res.writeHead(200, { "content-type": "application/json", ...corsHeaders });
    res.end(JSON.stringify({ ok: true }));
    return;
  }

  if (req.method !== "POST" || req.url !== "/execute") {
    res.writeHead(404, corsHeaders).end();
    return;
  }

  try {
    const body = await readBody(req);
    const payload = JSON.parse(body);

    const result = await executeEnvelope(payload);
    if (result.ok) {
      res.writeHead(200, { "content-type": "application/json", ...corsHeaders });
      res.end(JSON.stringify(result.data));
      return;
    }

    res.writeHead(400, { "content-type": "application/json", ...corsHeaders });
    res.end(JSON.stringify({
      code: result.error?.code ?? "execution_error",
      message: result.error?.message ?? "unknown bridge error",
      retryable: result.error?.retryable ?? false,
      fingerprint: result.error?.fingerprint ?? "bridge",
      request_id: result.error?.request_id ?? payload.request_id ?? "",
      command_id: result.error?.command_id ?? payload.command_id ?? "",
    }));
  } catch (err) {
    res.writeHead(500, { "content-type": "application/json", ...corsHeaders });
    res.end(
      JSON.stringify({
        code: "execution_error",
        message: `bridge failure: ${err instanceof Error ? err.message : String(err)}`,
        retryable: false,
        fingerprint: "bridge-failure",
        request_id: "",
        command_id: "",
      }),
    );
  }
});

server.listen(port, "127.0.0.1", () => {
  // eslint-disable-next-line no-console
  console.log(`[bridge] listening at http://127.0.0.1:${port}`);
});

function ensureBinary() {
  if (existsSync(binaryPath)) {
    return;
  }

  const build = spawnSync("cargo", ["build", "--manifest-path", manifestPath, "--bin", "gh-client-envelope-cli"], {
    stdio: "inherit",
  });

  if (build.status !== 0) {
    throw new Error("failed to build gh-client-envelope-cli");
  }
}

function readBody(req) {
  return new Promise((resolveBody, rejectBody) => {
    const chunks = [];
    req.on("data", (chunk) => chunks.push(Buffer.from(chunk)));
    req.on("end", () => resolveBody(Buffer.concat(chunks).toString("utf8")));
    req.on("error", rejectBody);
  });
}

function executeEnvelope(envelope) {
  return new Promise((resolveExec, rejectExec) => {
    const child = spawn(binaryPath, [], {
      stdio: ["pipe", "pipe", "pipe"],
      env: process.env,
    });

    let stdout = "";
    let stderr = "";

    child.stdout.on("data", (chunk) => {
      stdout += chunk.toString();
    });

    child.stderr.on("data", (chunk) => {
      stderr += chunk.toString();
    });

    child.on("error", rejectExec);

    child.on("close", (code) => {
      if (code !== 0 && !stdout.trim()) {
        rejectExec(new Error(`bridge command failed (${code}): ${stderr}`));
        return;
      }

      try {
        const parsed = JSON.parse(stdout.trim());
        resolveExec(parsed);
      } catch (err) {
        rejectExec(
          new Error(`failed to parse bridge response: ${err instanceof Error ? err.message : String(err)}; stderr=${stderr}`),
        );
      }
    });

    child.stdin.write(JSON.stringify(envelope));
    child.stdin.end();
  });
}
