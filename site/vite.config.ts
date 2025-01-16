import { fileURLToPath, URL } from "node:url";

import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import vueJsx from "@vitejs/plugin-vue-jsx";
import vueDevTools from "vite-plugin-vue-devtools";
import browserslistToEsbuild from "browserslist-to-esbuild";
import { ViteEjsPlugin } from 'vite-plugin-ejs'

// https://vitejs.dev/config/
export default defineConfig({
  build: {
    target: browserslistToEsbuild(undefined, {
      path: ".browserlistrc",
    }),
  },
  plugins: [vue(), vueJsx(), ViteEjsPlugin(), vueDevTools()],
  css: {
    preprocessorOptions: {
      scss: {
        api: "modern-compiler",
      },
    },
    devSourcemap: true,
  },
  resolve: {
    alias: {
      "@": fileURLToPath(new URL("./src", import.meta.url)),
    },
  },
});
