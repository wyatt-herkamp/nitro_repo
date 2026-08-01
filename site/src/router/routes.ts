import type { Component } from "vue";

import BrowseView from "@/views/BrowseView.vue";
import LoginView from "@/views/LoginView.vue";
import LogoutView from "@/views/LogoutView.vue";
import NotFound from "@/views/NotFound.vue";
import NpmLoginView from "@/views/NpmLoginView.vue";
import RepositoriesView from "@/views/RepositoriesView.vue";
import SearchView from "@/views/SearchView.vue";

import { adminRoutes } from "@/views/admin/adminRoutes";
import { profileRoutes } from "@/views/profile/profileRoutes";
import { projectRoutes } from "@/views/projects";
import { repositoryPages } from "@/views/repositoryPages";

declare module "vue-router" {
  interface RouteMeta {
    requiresAuth?: boolean;
    requiresRepositoryManager?: boolean;
    requiresUserManager?: boolean;
    sideBar?: Component;
    tag?: string;
    /**
     * Keeps the route out of the generated `routes.json`, and so out of the backend's SPA fallback
     * list. `/` is served directly and the catch-all would swallow every genuine 404.
     */
    skipRoutesJson?: boolean;
  }
}

/**
 * The route table, kept separate from `createRouter` so `scripts/generate-routes.mjs` can read it
 * without constructing a router — `createWebHistory` touches `window.history` and `location` the
 * moment it is called, which a build-time script has neither of.
 */
export const routes = [
  {
    path: "/",
    name: "home",
    // `HomeView.vue` was a byte-identical copy of `RepositoriesView.vue`; both routes render the one
    // component now.
    component: RepositoriesView,
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
    // Where `npm login` sends the browser. The registry hands npm this URL as `loginUrl`.
    path: "/npm/login/:session",
    name: "npmLogin",
    component: NpmLoginView,
  },
  {
    path: "/search",
    name: "search",
    component: SearchView,
  },
  ...repositoryPages,
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
