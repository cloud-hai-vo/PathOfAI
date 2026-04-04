import { defineConfig } from "vite";

export default defineConfig({
  // Tauri dev server runs on 5173
  server: {
    port: 5173,
    strictPort: true,
    host: "localhost",
    hmr: { protocol: "ws", host: "localhost" },
  },
  // Build output goes to src/ directory (Tauri reads frontendDist: "../src")
  build: {
    outDir: "src",
    emptyOutDir: false,  // don't delete our .ts/.css source files
    target: ["es2021", "chrome100", "safari13"],
    minify: !process.env.TAURI_DEBUG,
    sourcemap: !!process.env.TAURI_DEBUG,
  },
  envPrefix: ["VITE_", "TAURI_"],
});
