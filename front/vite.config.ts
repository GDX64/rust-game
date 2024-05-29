import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

declare const process: any;

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [vue()],
  define: {
    IS_PROD: process.env.NODE_ENV === "production",
  },
  server: {
    open: true,
  },
});

