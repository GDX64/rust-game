import { defineConfig } from "vite";
import wasm from "vite-plugin-wasm";
import dts from "vite-plugin-dts";

declare const process: any;

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    wasm(),
    dts({
      include: "js/libInterface.ts",
    }),
  ],
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
    lib: {
      entry: "./js/lib.ts",
      formats: ["es"],
      fileName: "lib",
    },
  },
  base: "/static/editor/",
});

