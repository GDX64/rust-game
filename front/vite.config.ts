import { defineConfig } from "vite";
import wasm from "vite-plugin-wasm";
import dts from "vite-plugin-dts";
import vue from "@vitejs/plugin-vue";

declare const process: any;

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    wasm(),
    dts({
      include: "js/libInterface.ts",
    }),
    vue(),
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
    target: "es2022",
    minify: true,
    // lib: {
    //   entry: "./js/lib.ts",
    //   formats: ["es"],
    //   fileName: "lib",
    // },
  },
  base: "/static/",
});
