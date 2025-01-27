import { ApacheMavenIcon, GradleIcon } from "vue3-simple-icons";
import MavenProjectHelper from "./MavenProjectHelper.vue";
import type { FrontendRepositoryType } from "@/types/repository";
import MavenFullProject from "./MavenFullProject.vue";
export const MavenFrontendDefinition = {
  name: "maven",
  properName: "Maven",
  projectComponent: {
    component: MavenProjectHelper,
  },
  fullProjectComponent: {
    component: MavenFullProject,
  },
  icons: [
    {
      name: "Apache Maven",
      component: ApacheMavenIcon,
      url: "https://maven.apache.org/",
      props: {},
    },
    {
      name: "Gradle",
      component: GradleIcon,
      url: "https://gradle.org/",
      props: {},
    },
  ],
} as FrontendRepositoryType;
export interface MavenProxyRoute {
  url: string;
  name?: string;
}
export interface MavenProxyConfigType {
  routes: MavenProxyRoute[];
}
export function defaultProxy(): MavenProxyConfigType {
  return {
    routes: [],
  };
}
export type MavenConfigType =
  | {
      type: "Hosted";
    }
  | {
      type: "Proxy";
      config: MavenProxyConfigType;
    };
