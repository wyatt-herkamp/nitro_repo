import { createApp } from "vue";
import App from "./App.vue";
import router from "./router";
import { VueCookieNext } from "vue-cookie-next";

import ElementPlus from "element-plus";
import "element-plus/dist/index.css";
const app = createApp(App);
app.use(router);
app.use(VueCookieNext);
app.use(ElementPlus);
app.mount("#app");

// set default config
VueCookieNext.config({ expire: "7d" });
