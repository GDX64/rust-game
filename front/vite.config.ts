import { defineConfig } from "vite";
import wasm from "vite-plugin-wasm";
import vue from "@vitejs/plugin-vue";

declare const process: any;

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [vue(), wasm()],
  define: {
    FRONT_SERVER: `"${process.env.FRONT_SERVER ?? "ws://localhost:5000/ws"}"`,
  },
  server: {
    open: true,
  },
  build: {
    target: "esnext",
  },
  base: "/game/",
});

