import "./assets/styles/main.scss";
import "vue-final-modal/style.css";
import piniaPluginPersistedstate from "pinia-plugin-persistedstate";
import Notifications from "@kyvg/vue3-notification";

import { createApp } from "vue";
import { createPinia } from "pinia";
import { createVfm } from "vue-final-modal";
import App from "./App.vue";
import router from "./router";
import { createMetaManager } from "vue-meta";
import { FontAwesomeIcon } from "@fortawesome/vue-fontawesome";
import { library } from "@fortawesome/fontawesome-svg-core";
import {
  faCalendar,
  faFileImage,
  faFileText,
  faGear,
  faUser,
  faBars,
  faX,
  faRightToBracket,
  faPenToSquare,
  faFloppyDisk,
  faArrowLeft,
  faHome,
  faArrowRight,
  faAnglesRight,
  faAnglesLeft,
  faEye,
  faEyeSlash,
  faUsers,
  faBoxOpen,
  faBoxesPacking,
  faToolbox,
  faUserPlus,
  faAngleDown,
  faCircleXmark,
  faCheckCircle,
  faFile,
  faFolder,
} from "@fortawesome/free-solid-svg-icons";

import { sessionStore } from "./stores/session";
import { autoAnimatePlugin } from "@formkit/auto-animate/vue";

const app = createApp(App);
const vfm = createVfm();
router.beforeEach((to) => {
  const store = sessionStore(pinia);
  const signedIn = store.session !== undefined;
  if ((to.meta.requiresAuth || to.meta.requiresIdentity === true) && !signedIn) {
    // Carry where they were going, so signing in lands them there rather than on the home page.
    return { name: "login", query: { redirect: to.fullPath } };
  }
  // `requiresUserManager` and `requiresRepositoryManager` were declared on routes and in the
  // `RouteMeta` augmentation, and nothing ever read them — every admin route rendered for any
  // signed-in user. The server is still the real check; this stops the UI from offering pages it
  // knows will be refused.
  if (to.meta.requiresUserManager || to.meta.requiresRepositoryManager) {
    if (!signedIn) {
      return { name: "login", query: { redirect: to.fullPath } };
    }
    const user = store.user;
    const permitted =
      user !== undefined &&
      (user.admin ||
        (to.meta.requiresUserManager === true && user.user_manager) ||
        (to.meta.requiresRepositoryManager === true && user.system_manager));
    if (!permitted) {
      return { name: "home" };
    }
  }
});

app.use(router);

/* add icons to the library */
library.add(faGear);
library.add(faUser);
library.add(faBars);
library.add(faFileText);
library.add(faFileImage);
library.add(faCalendar);
library.add(faRightToBracket);
library.add(faX);
library.add(faPenToSquare);
library.add(faFloppyDisk);
library.add(faArrowLeft);
library.add(faHome);
library.add(faArrowRight);
library.add(faAnglesRight);
library.add(faAnglesLeft);
library.add(faAngleDown);
library.add(faEye);
library.add(faEyeSlash);
library.add(faUsers);
library.add(faBoxOpen);
library.add(faBoxesPacking);
library.add(faToolbox);
library.add(faUserPlus);
library.add(faCircleXmark);
library.add(faCheckCircle);
library.add(faFile);
library.add(faFolder);
const pinia = createPinia();
pinia.use(piniaPluginPersistedstate);
app.use(createMetaManager());
app.use(pinia);
app.component("font-awesome-icon", FontAwesomeIcon);
app.use(Notifications);
app.use(autoAnimatePlugin);
app.use(vfm);
app.mount("#app");
