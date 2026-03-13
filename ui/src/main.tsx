import React from "react";
import ReactDOM from "react-dom/client";
import { BrowserRouter } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

import App from "./App";
import { I18nProvider } from "./core/i18n";
import "./styles.css";

const queryClient = new QueryClient();
const rootElement = document.getElementById("root");
const startupTimestamp = Date.now();
const MIN_SPLASH_DISPLAY_MS = 900;

if (!rootElement) {
  throw new Error("root element not found");
}

ReactDOM.createRoot(rootElement).render(
  <React.StrictMode>
    <QueryClientProvider client={queryClient}>
      <I18nProvider>
        <BrowserRouter>
          <App />
        </BrowserRouter>
      </I18nProvider>
    </QueryClientProvider>
  </React.StrictMode>,
);

const closeDesktopSplashscreen = async () => {
  const scopedWindow = window as unknown as { __TAURI_INTERNALS__?: unknown };
  if (!scopedWindow.__TAURI_INTERNALS__) {
    return;
  }

  try {
    const elapsed = Date.now() - startupTimestamp;
    const waitMs = Math.max(0, MIN_SPLASH_DISPLAY_MS - elapsed);
    if (waitMs > 0) {
      await new Promise<void>((resolve) => {
        window.setTimeout(resolve, waitMs);
      });
    }

    const { invoke } = await import("@tauri-apps/api/core");
    await invoke("close_splashscreen");
  } catch (error) {
    console.warn("failed to close splashscreen", error);
  }
};

if (typeof window.requestAnimationFrame === "function") {
  window.requestAnimationFrame(() => {
    void closeDesktopSplashscreen();
  });
} else {
  void closeDesktopSplashscreen();
}
