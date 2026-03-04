import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";

import { describe, expect, it } from "vitest";

import { COMMAND_CATALOG } from "./commandCatalog";
import { STABLE_COMMAND_IDS } from "./commandIds";

function loadRustStableCommandIds(): string[] {
  const candidates = [
    resolve(process.cwd(), "../src-tauri/src/contract/mod.rs"),
    resolve(process.cwd(), "src-tauri/src/contract/mod.rs"),
  ];

  const path = candidates.find((candidate) => existsSync(candidate));
  if (!path) {
    throw new Error("unable to locate src-tauri/src/contract/mod.rs");
  }

  const source = readFileSync(path, "utf8");
  const blockMatch = source.match(/pub const STABLE_COMMAND_IDS:[\s\S]*?=\s*&\[(?<block>[\s\S]*?)\];/m);
  if (!blockMatch?.groups?.block) {
    throw new Error("failed to parse STABLE_COMMAND_IDS block");
  }

  const ids = [...blockMatch.groups.block.matchAll(/"([^"]+)"/g)].map((match) => match[1]);
  return ids;
}

describe("command catalog", () => {
  it("matches Rust stable command ids exactly", () => {
    const rustIds = loadRustStableCommandIds().sort();
    const frontendIds = [...STABLE_COMMAND_IDS].sort();

    expect(frontendIds).toEqual(rustIds);
  });

  it("has spec metadata for every command", () => {
    for (const id of STABLE_COMMAND_IDS) {
      const spec = COMMAND_CATALOG[id];
      expect(spec).toBeDefined();
      expect(spec.payloadSchema).toBeDefined();
      expect(spec.responseSchema).toBeDefined();
      expect(["viewer", "write", "admin"]).toContain(spec.requiredPermission);
      expect(["screen", "console"]).toContain(spec.exposure);
    }
  });
});
