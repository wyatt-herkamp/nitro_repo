/* eslint-disable */
import { createApp } from "vue";
import App from "./App.vue";
import router from "./router";
import { VueCookieNext } from "vue-cookie-next";
import Notifications from "@kyvg/vue3-notification";
import { createMetaManager } from "vue-meta";
import "@/styles/app.css";
import "@/styles/form.css";
import "boxicons/css/boxicons.min.css";
import "boxicons/dist/boxicons.js";
import VueUploadComponent from "vue-upload-component";
import vfmPlugin from "vue-final-modal";
import { init } from "nitro_repo-api-wrapper";
import { apiURL } from "@/http-common";

init(apiURL);
const app = createApp(App);
app.use(VueCookieNext);
app.use(Notifications);
app.use(router);
app.use(vfmPlugin);
app.use(createMetaManager());

app.component("file-upload", VueUploadComponent);

app.mount("#app");

// set default config
VueCookieNext.config({ expire: "7d" });
