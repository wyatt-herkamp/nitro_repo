// @ts-ignore
import path from "path";
import {defineConfig} from "vite";
import vue from "@vitejs/plugin-vue";

import Components from "unplugin-vue-components/vite";
import {ElementPlusResolver} from "unplugin-vue-components/resolvers";
import {ViteEjsPlugin} from "vite-plugin-ejs";

// https://vitejs.dev/config/
export default defineConfig({
    resolve: {
        alias: {
            "~/": `${path.resolve(__dirname, "src")}/`,
            "@/": `${path.resolve(__dirname, "src")}/`,
        },
    },
    css: {
        preprocessorOptions: {
            scss: {
                additionalData: `@use "~/styles/element/index.scss" as *;`,
            },
        },
    },
    plugins: [
        vue(),
        ViteEjsPlugin(),

        Components({
            resolvers: [
                ElementPlusResolver({
                    importStyle: "sass",
                }),
            ],
        }),
    ],
});
