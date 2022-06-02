import { createRouter, createWebHistory, RouteRecordRaw } from "vue-router";
import Home from "../views/Home.vue";
import Admin from "../views/admin/Admin.vue";
import ViewRepository from "../views/admin/ViewRepository.vue";
import ViewStorage from "../views/admin/ViewStorage.vue";
import ViewUser from "../views/admin/ViewUser.vue";
import Browse from "../views/Browse.vue";
import Upload from "../views/Upload.vue";
import Project from "../views/Project.vue";
import Repository from "../views/Repository.vue";
import Me from "../views/Me.vue";
import Login from "../views/Login.vue";
import { useCookies } from "vue3-cookies";
import { inject } from "vue";

const routes: Array<RouteRecordRaw> = [
  {
    path: "/",
    name: "Home",
    component: Home,
  },
  {
    path: "/me",
    name: "Me",
    component: Me,
    beforeEnter(to, from) {
      return requireAuth(to, from);
    },
  },
  {
    path: "/login",
    name: "Login",
    component: Login,
  },

  {
    path: "/admin/:page?",
    name: "Admin",
    component: Admin,
    beforeEnter(to, from) {
      return requireAuth(to, from);
    },
  },
  {
    path: "/admin/repository/:storage/:repo",
    name: "AdminRepoView",
    component: ViewRepository,
    beforeEnter(to, from) {
      return requireAuth(to, from);
    },
  },
  {
    path: "/admin/storage/:storage",
    name: "AdminStorageView",
    component: ViewStorage,
    beforeEnter(to, from) {
      return requireAuth(to, from);
    },
  },
  {
    path: "/admin/user/:user",
    name: "AdminUserView",
    component: ViewUser,
    beforeEnter(to, from) {
      return requireAuth(to, from);
    },
  },
  {
    path: "/upload/:storage/:repo",
    name: "Upload",
    component: Upload,
    beforeEnter(to, from) {
      return requireAuth(to, from);
    },
  },

  {
    path: "/browse/:catchAll(.*)?",
    name: "Browse",
    component: Browse,
  },
  {
    path: "/repository/:storage/:repo",
    name: "ViewRepository",
    component: Repository,
  },

  {
    path: "/project/:storage/:repo/:id/:version?",
    name: "Project",
    component: Project,
  },
];

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes,
});
function requireAuth(to: any, from: any) {
  const { cookies } = useCookies();
  if (cookies.get("token") == undefined) {
    return `login?return=${to.fullPath}`;
  }
  return true;
}
export default router;
