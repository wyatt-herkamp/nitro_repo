import { DockerIcon } from "vue3-simple-icons";
import DockerProjectHelper from "./DockerProjectHelper.vue";
import type { FrontendRepositoryType } from "@/types/repository";

export const DockerFrontendDefinition = {
  name: "docker",
  properName: "Docker",
  projectComponent: {
    component: DockerProjectHelper,
    props: {},
  },
  icons: [
    {
      name: "Docker",
      component: DockerIcon,
      url: "https://docs.docker.com/registry/spec/api/",
      props: {},
    },
  ],
} as FrontendRepositoryType;

/**
 * Mirrors `DockerRegistryConfig` on the backend. Only `Hosted` exists; a `Proxy` variant (a
 * pull-through cache) would be added here at the same time as the server's.
 */
export type DockerConfigType = {
  type: "Hosted";
};
