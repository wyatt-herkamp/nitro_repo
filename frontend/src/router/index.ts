import { createRouter, createWebHistory, RouteRecordRaw } from "vue-router";
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
import Index from "../views/Index.vue";
import { useCookies } from "vue3-cookies";

const routes: Array<RouteRecordRaw> = [
  {
    path: "/",
    name: "Index",
    component: Index,
  },
  {
    path: "/me",
    name: "Me",
    component: Me,
    beforeEnter(to) {
      return requireAuth(to);
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
    beforeEnter(to) {
      return requireAuth(to);
    },
  },
  {
    path: "/admin/repository/:storage/:repo",
    name: "AdminRepoView",
    component: ViewRepository,
    beforeEnter(to) {
      return requireAuth(to);
    },
  },
  {
    path: "/admin/storage/:storage",
    name: "AdminStorageView",
    component: ViewStorage,
    beforeEnter(to) {
      return requireAuth(to);
    },
  },
  {
    path: "/admin/user/:user",
    name: "AdminUserView",
    component: ViewUser,
    beforeEnter(to) {
      return requireAuth(to);
    },
  },
  {
    path: "/upload/:storage/:repo",
    name: "Upload",
    component: Upload,
    beforeEnter(to) {
      return requireAuth(to);
    },
  },

  {
    path: "/browse/:storage/:repo/:catchAll(.*)?",
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
function requireAuth(to: { fullPath: string }) {
  const { cookies } = useCookies();
  if (cookies.get("logged_in") === undefined) {
    return `login?return=${to.fullPath}`;
  }
  return true;
}
const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes,
});
export default router;
