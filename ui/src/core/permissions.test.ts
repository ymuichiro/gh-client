import { describe, expect, it } from "vitest";

import { normalizeViewerPermission, resolveEnvelopePermission } from "./permissions";

describe("permissions", () => {
  it("normalizes GitHub permission labels", () => {
    expect(normalizeViewerPermission("ADMIN")).toBe("admin");
    expect(normalizeViewerPermission("WRITE")).toBe("write");
    expect(normalizeViewerPermission("READ")).toBe("viewer");
    expect(normalizeViewerPermission("unknown")).toBeNull();
  });

  it("requires context for write/admin when needed", () => {
    expect(resolveEnvelopePermission("viewer", null, true)).toBe("viewer");
    expect(resolveEnvelopePermission("write", null, true)).toBeNull();
    expect(resolveEnvelopePermission("admin", "write", true)).toBeNull();
    expect(resolveEnvelopePermission("write", "admin", true)).toBe("write");
    expect(resolveEnvelopePermission("admin", "admin", true)).toBe("admin");
  });

  it("returns required permission directly when repo context is not needed", () => {
    expect(resolveEnvelopePermission("write", null, false)).toBe("write");
    expect(resolveEnvelopePermission("admin", null, false)).toBe("admin");
  });
});
