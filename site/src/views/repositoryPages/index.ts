import RepositoryPageView from "./RepositoryPageView.vue";

export const repositoryPages = [
  {
    path: "/page/repository/:repositoryId",
    name: "repository_page_by_id",
    component: RepositoryPageView,
  },
  {
    path: "/page/repository/:storageName/:repositoryName",
    name: "repository_page_by_name",
    component: RepositoryPageView,
  },
];
