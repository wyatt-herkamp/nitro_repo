import { createApp } from "vue";
import App from "./App.vue";
import router from "./router";
import { VueCookieNext } from "vue-cookie-next";
import Notifications from '@kyvg/vue3-notification'

import ElementPlus from "element-plus";
import "element-plus/dist/index.css";
const app = createApp(App);
app.use(router);
app.use(VueCookieNext);
app.use(ElementPlus);
app.use(Notifications)
app.mount("#app");

// set default config
VueCookieNext.config({ expire: "7d" });
