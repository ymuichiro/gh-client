import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { fileURLToPath } from "node:url";

const mainEntry = fileURLToPath(new URL("./index.html", import.meta.url));
const splashEntry = fileURLToPath(new URL("./splashscreen.html", import.meta.url));

export default defineConfig({
  base: "./",
  plugins: [react()],
  build: {
    rollupOptions: {
      input: {
        main: mainEntry,
        splashscreen: splashEntry,
      },
    },
  },
  server: {
    host: "127.0.0.1",
    port: 5173,
  },
});
