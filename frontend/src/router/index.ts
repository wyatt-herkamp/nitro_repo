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
  }, {
    path: "/login",
    name: "Login",
    component: Login,
  },

  {
    path: "/admin/:page?",
    name: "Admin",
    component: Admin,
  },
  {
    path: "/admin/repository/:storage/:repo",
    name: "AdminRepoView",
    component: ViewRepository,
  },
  {
    path: "/admin/storage/:storage",
    name: "AdminStorageView",
    component: ViewStorage,
  },
  {
    path: "/admin/user/:user",
    name: "AdminUserView",
    component: ViewUser,
  },
  {
    path: "/upload/:storage/:repo",
    name: "Upload",
    component: Upload,
  },

  {
    path: "/browse/:catchAll(.*)?",
    name: "Browse",
    component: Browse,
  },
  {
    path: "/repository/:storage/:repo/",
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

export default router;
