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
  priority?: number;
  username?: string;
  password?: string;
}
export interface MavenProxyConfigType {
  routes: MavenProxyRoute[];
  /** Seconds a cached artifact stays valid. 0 keeps it forever. */
  cache_ttl_seconds: number;
  /** Seconds a cached maven-metadata.xml or snapshot build stays valid. */
  mutable_ttl_seconds: number;
}
export function defaultProxy(): MavenProxyConfigType {
  return {
    routes: [],
    // Released artifacts are immutable by convention, so they are kept indefinitely.
    cache_ttl_seconds: 0,
    // Metadata and snapshots change upstream, so they are re-checked every 15 minutes.
    mutable_ttl_seconds: 900,
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
