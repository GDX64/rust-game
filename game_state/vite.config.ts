import { defineConfig } from "vite";
import wasm from "vite-plugin-wasm";

declare const process: any;

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [wasm()],
  define: {
    IS_PROD: process.env.NODE_ENV === "production",
  },
  server: {
    open: true,
  },
  optimizeDeps: {
    exclude: ["three"],
  },
  build: {
    target: "esnext",
  },
  base: "/static/editor/",
});

