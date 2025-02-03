import ProjectPageView from "./ProjectPageView.vue";

export const projectRoutes = [
  {
    path: "/projects/:projectId/:version?",

    name: "ProjectPageView",
    component: ProjectPageView,
  },
  {
    path: "/projects/:storageName/:repositoryName/:projectKey/:version?",
    name: "project-page-by-key",
    component: ProjectPageView,
  },
];
