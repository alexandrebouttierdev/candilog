/// <reference types="vitest/config" />
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import { fileURLToPath, URL } from "node:url";

// Tauri sert le frontend sur un port fixe et échoue si celui-ci est déjà pris :
// `strictPort` transforme un port occupé en erreur explicite au lieu d'un décalage
// silencieux que la fenêtre native ne suivrait pas.
export default defineConfig({
  plugins: [react(), tailwindcss()],
  resolve: {
    alias: {
      "@": fileURLToPath(new URL("./src", import.meta.url)),
    },
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: { ignored: ["**/src-tauri/**"] },
  },
  test: {
    environment: "jsdom",
    setupFiles: ["./src/shared/lib/test-setup.ts"],
    globals: true,
  },
});
