import { fileURLToPath, URL } from "node:url";

import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import vueJsx from "@vitejs/plugin-vue-jsx";
import vueDevTools from "vite-plugin-vue-devtools";
import browserslistToEsbuild from "browserslist-to-esbuild";
import { ViteEjsPlugin } from "vite-plugin-ejs";
import fs from "node:fs";
import { execFileSync } from "node:child_process";
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
      // Regenerates routes.json from the router before the build, then ships it. It used to only
      // copy a hand-maintained file, so the SPA fallback list drifted from the actual routes and a
      // missing entry meant a hard 404 on refresh.
      name: "routes-json",
      apply: "build",

      buildStart() {
        // In its own process on purpose. The generator spins up a throwaway Vite server to load the
        // route table, and closing that server disposes the Sass compiler this build shares with it
        // — every stylesheet afterwards then fails with "Cannot read properties of undefined".
        execFileSync(process.execPath, ["scripts/generate-routes.mjs"], {
          cwd: fileURLToPath(new URL(".", import.meta.url)),
          stdio: "inherit",
        });
      },

      closeBundle() {
        fs.copyFileSync(
          fileURLToPath(new URL("./src/router/routes.json", import.meta.url)),
          fileURLToPath(new URL("./dist/routes.json", import.meta.url)),
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
