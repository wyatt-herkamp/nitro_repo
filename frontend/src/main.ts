/* eslint-disable */
import { createApp } from "vue";
import App from "./App.vue";
import router from "./router";
import Notifications from "@kyvg/vue3-notification";
import { createMetaManager } from "vue-meta";
import "@/styles/app.css";
import "@/styles/form.css";
import "boxicons/css/boxicons.min.css";
import "boxicons/dist/boxicons.js";
import vfmPlugin from "vue-final-modal";
import { init } from "@nitro_repo/nitro_repo-api-wrapper";
import { apiURL } from "@/http-common";
import { useCookies } from "vue3-cookies";
import { createPinia } from "pinia";

init(apiURL);

const { cookies } = useCookies();

const app = createApp(App);
app.use(Notifications);
app.use(router);
app.use(vfmPlugin);
app.use(createMetaManager());
app.use(createPinia());

app.provide("token", cookies.get("token"));

app.mount("#app");
