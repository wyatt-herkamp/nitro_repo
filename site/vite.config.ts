import { fileURLToPath, URL } from "node:url";

import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import vueJsx from "@vitejs/plugin-vue-jsx";
import vueDevTools from "vite-plugin-vue-devtools";
import browserslistToEsbuild from "browserslist-to-esbuild";
import { ViteEjsPlugin } from "vite-plugin-ejs";
import fs from "fs";
// https://vitejs.dev/config/
export default defineConfig({
  build: {
    target: browserslistToEsbuild(undefined, {
      path: ".browserlistrc",
    }),
  },
  plugins: [
    vue(),
    vueJsx(),
    ViteEjsPlugin(),
    vueDevTools(),
    {
      name: "copy-routes",
      apply: "build",

      closeBundle() {
        console.log("Copying routes.json to dist/assets");
        fs.copyFile(
          fileURLToPath(new URL("./src/router/routes.json", import.meta.url)),
          fileURLToPath(new URL("./dist/routes.json", import.meta.url)),
          (err) => {
            if (err) {
              console.error(err);
            } else {
              console.log("routes.json copied successfully");
            }
          },
        );
      },
    },
  ],
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
