/* eslint-disable */
import { createApp } from "vue";
import App from "./App.vue";
import router from "./router";
import { useCookie, VueCookieNext } from "vue-cookie-next";
import Notifications from "@kyvg/vue3-notification";
import { createMetaManager } from "vue-meta";
import "@/styles/app.css";
import "@/styles/form.css";
import "boxicons/css/boxicons.min.css";
import "boxicons/dist/boxicons.js";
import vfmPlugin from "vue-final-modal";
import { init } from "nitro_repo-api-wrapper";
import { apiURL } from "@/http-common";
import { useCookies } from "vue3-cookies";

init(apiURL);
const { cookies } = useCookies();

const app = createApp(App);
app.use(VueCookieNext);
app.use(Notifications);
app.use(router);
app.use(vfmPlugin);
app.use(createMetaManager());
app.provide("token", cookies.get("token")
)


app.mount("#app");

// set default config
VueCookieNext.config({ expire: "7d" });
