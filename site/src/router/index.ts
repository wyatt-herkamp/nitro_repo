import { createRouter, createWebHistory } from "vue-router";
import HomeView from "../views/HomeView.vue";
import BrowseView from "@/views/BrowseView.vue";
import LoginView from "@/views/LoginView.vue";
import LogoutView from "@/views/LogoutView.vue";

import RepositoriesView from "@/views/RepositoriesView.vue";
import RepositoryPageView from "@/views/RepositoryPageView.vue";
import type { Component } from "vue";

import { adminRoutes } from "@/views/admin/adminRoutes";
import { profileRoutes } from "@/views/profile/profileRoutes";
import { projectRoutes } from "@/views/projects";
import NotFound from "@/views/NotFound.vue";
declare module "vue-router" {
  interface RouteMeta {
    requiresAuth?: boolean;
    requiresRepositoryManager?: boolean;
    requiresUserManager?: boolean;
    sideBar?: Component;
    tag?: string;
    skipRoutesJson?: boolean;
  }
}
const routes = [
  {
    path: "/",
    name: "home",
    component: HomeView,
    meta: {
      skipRoutesJson: true,
    },
  },

  {
    path: "/browse/:id/:catchAll(.*)?",
    name: "Browse",
    component: BrowseView,
  },

  {
    path: "/login",
    name: "login",
    component: LoginView,
  },
  {
    path: "/logout",
    name: "logout",
    component: LogoutView,
  },
  {
    path: "/page/repositories",
    name: "repositories",
    component: RepositoriesView,
  },
  {
    path: "/page/repository/:id",
    name: "repository",
    component: RepositoryPageView,
  },
  ...adminRoutes,
  ...profileRoutes,
  ...projectRoutes,
  {
    path: "/:pathMatch(.*)*",
    name: "not-found",
    component: NotFound,
    meta: {
      skipRoutesJson: true,
    },
  },
];
const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: routes,
});

export default router;
