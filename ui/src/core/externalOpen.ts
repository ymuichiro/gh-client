function hasTauriRuntime(): boolean {
  if (typeof window === "undefined") {
    return false;
  }
  const scopedWindow = window as unknown as { __TAURI_INTERNALS__?: unknown };
  return Boolean(scopedWindow.__TAURI_INTERNALS__);
}

export async function openExternalUrl(url: string): Promise<void> {
  if (!url) {
    return;
  }

  if (hasTauriRuntime()) {
    try {
      const { openUrl } = await import("@tauri-apps/plugin-opener");
      await openUrl(url);
      return;
    } catch {
      // Fall through to browser open path in case Tauri opener is unavailable.
    }
  }

  if (typeof window !== "undefined") {
    window.open(url, "_blank", "noopener,noreferrer");
  }
}
