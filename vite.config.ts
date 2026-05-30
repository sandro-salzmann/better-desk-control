import { fileURLToPath } from "node:url";
import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

const entry = (file: string) => fileURLToPath(new URL(file, import.meta.url));

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(async () => ({
  plugins: [react(), tailwindcss()],

  // Multi-page build: the desk app (`index.html`) plus a standalone component
  // gallery (`components.html`). The gallery is never linked from the app.
  build: {
    rollupOptions: {
      input: {
        main: entry("index.html"),
        components: entry("components.html"),
      },
    },
  },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell Vite to ignore the Rust source and the shared Cargo `target`
      //    dir (now at the repo root since the crates share one workspace).
      //    Watching `target` crashes the dev server with EBUSY on Windows
      //    because Cargo locks the built `.dll` while the app runs.
      ignored: ["**/src-tauri/**", "**/target/**"],
    },
  },
}));
