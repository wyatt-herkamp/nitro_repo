import {createRouter, createWebHistory, RouteRecordRaw} from "vue-router";
import Home from "../views/Home.vue";
import Install from "../views/Install.vue";
import Admin from "../views/admin/Admin.vue";
import Storages from "../views/admin/Storages.vue";
import Users from "../views/admin/Users.vue";
import Repositories from "../views/admin/Repositories.vue";
import ViewRepository from "../views/admin/ViewRepository.vue";
import Settings from "../views/admin/Settings.vue";
import Browse from "../views/Browse.vue";
import Upload from "../views/Upload.vue";
import Project from "../views/Project.vue";
import Me from "../views/Me.vue";

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
  },
  {
    path: "/install",
    name: "Install",
    component: Install,
  },
  {
    path: "/admin",
    name: "Admin",
    component: Admin,
  },
  {
    path: "/admin/settings",
    name: "Settings",
    component: Settings,
  },
  {
    path: "/admin/repositories",
    name: "AdminRepos",
    component: Repositories,
  },
  {
    path: "/admin/repository/:repo",
    name: "AdminRepoView",
    component: ViewRepository,
  },
  {
    path: "/admin/users",
    name: "AdminUsers",
    component: Users,
  },
  {
    path: "/admin/storages",
    name: "AdminStorages",
    component: Storages,
  },
  {
    path: "/upload/:storage/:repo",
    name: "Upload",
    component: Upload,
  },
  {
    path: "/browse/:storage?/:repo?/:catchAll(.*)?",
    name: "Browse",
    component: Browse,
  },
  {
    path: "/browse/:storage/:repo/:catchAll(.*)",
    name: "Project",
    component: Project,
  },
];

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes,
});

export default router;
