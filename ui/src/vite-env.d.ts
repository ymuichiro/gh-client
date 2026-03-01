/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_EXECUTION_MODE?: "tauri" | "mock" | "bridge";
  readonly VITE_EXECUTION_BRIDGE_URL?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
