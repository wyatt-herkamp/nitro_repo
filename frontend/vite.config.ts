import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

import { ViteEjsPlugin } from "vite-plugin-ejs";
import { fileURLToPath, URL } from "node:url";

// https://vitejs.dev/config/
export default defineConfig({
  resolve: {
    alias: {
      "@": fileURLToPath(new URL("./src", import.meta.url)),
    },
  },
  css: {
    preprocessorOptions: {
      scss: {
        additionalData: `@import "@/styles/variables.scss";`,
      },
    },
  },
  plugins: [
    vue({
      template: {
        compilerOptions: {},
      },
    }),
    ViteEjsPlugin(),
  ],
});
