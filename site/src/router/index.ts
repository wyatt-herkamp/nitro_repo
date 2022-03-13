import {createRouter, createWebHistory, RouteRecordRaw} from "vue-router";
import Home from "../views/Home.vue";
import Install from "../views/Install.vue";
import Admin from "../views/admin/Admin.vue";
import ViewRepository from "../views/admin/ViewRepository.vue";
import Browse from "../views/Browse.vue";
import Upload from "../views/Upload.vue";
import Project from "../views/Project.vue";
import Repository from "../views/Repository.vue";
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
        path: "/admin/:page?",
        name: "Admin",
        component: Admin,
    },
  {
    path: "/admin/repository/:repo",
    name: "AdminRepoView",
    component: ViewRepository,
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
        path: "/repository/:storage/:repo/",
        name: "ViewRepository",
        component: Repository,
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
