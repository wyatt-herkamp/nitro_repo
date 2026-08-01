import { NpmIcon } from "vue3-simple-icons";
import NPMProjectHelper from "./NPMProjectHelper.vue";
import type { FrontendRepositoryType } from "@/types/repository";

/**
 * This was exported as `MavenFrontendDefinition` — a copy-paste from `maven.ts` — and was never
 * imported anywhere, so `repositoryTypes` listed only Maven. `findRepositoryType("npm")` returned
 * `undefined`, which is why every npm project page rendered "This repository has not been defined
 * in the frontend".
 */
export const NPMFrontendDefinition = {
  name: "npm",
  properName: "npm",
  projectComponent: {
    component: NPMProjectHelper,
    props: {},
  },
  icons: [
    {
      name: "npm",
      component: NpmIcon,
      url: "https://www.npmjs.com/",
      props: {},
    },
  ],
} as FrontendRepositoryType;

export interface NPMProxyRoute {
  url: string;
  name?: string;
}
export interface NPMProxyConfigType {
  routes: NPMProxyRoute[];
}
export function defaultProxy(): NPMProxyConfigType {
  return {
    routes: [],
  };
}
export type NPMConfigType =
  | {
      type: "Hosted";
    }
  | {
      type: "Proxy";
      config: NPMProxyConfigType;
    };
