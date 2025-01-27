import ProjectPageView from "./ProjectPageView.vue";

export const projectRoutes = [
  {
    path: "/projects/:projectId/:version?",

    name: "ProjectPageView",
    component: ProjectPageView,
  },
];
