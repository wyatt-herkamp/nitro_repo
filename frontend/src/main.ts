/* eslint-disable */
import { createApp } from "vue";
import App from "./App.vue";
import router from "./router";
import Notifications from "@kyvg/vue3-notification";
import { createMetaManager } from "vue-meta";
import "@/styles/app.css";
import "@/styles/form.css";
import vfmPlugin from "vue-final-modal";
import { createPinia } from "pinia";
/* import the fontawesome core */
import { library } from "@fortawesome/fontawesome-svg-core";

/* import font awesome icon component */
import { FontAwesomeIcon } from "@fortawesome/vue-fontawesome";

/* import specific icons */
import {
  faEye,
  faEyeSlash,
  faUserSecret,
} from "@fortawesome/free-solid-svg-icons";

/* add icons to the library */
library.add(faEyeSlash);
library.add(faEye);
const app = createApp(App);
app.use(Notifications);
app.use(router);
app.use(vfmPlugin);
app.use(createMetaManager());
app.use(createPinia());
app.component("font-awesome-icon", FontAwesomeIcon);

app.mount("#app");
