import { ApacheMavenIcon, GradleIcon } from 'vue3-simple-icons'
import MavenProjectHelper from './MavenProjectHelper.vue'
export const MavenFrontendDefinition = {
  name: 'maven',
  properName: 'Maven',
  projectComponent: {
    component: MavenProjectHelper,
    props: {}
  },
  icons: [
    {
      name: 'Apache Maven',
      component: ApacheMavenIcon,
      url: 'https://maven.apache.org/',
      props: {}
    },
    {
      name: 'Gradle',
      component: GradleIcon,
      url: 'https://gradle.org/',
      props: {}
    }
  ]
}
export interface MavenProxyRoute {
  url: string
  name?: string
}
export interface MavenProxyConfigType {
  routes: MavenProxyRoute[]
}
export function defaultProxy(): MavenProxyConfigType {
  return {
    routes: []
  }
}
export type MavenConfigType =
  | {
      type: 'Hosted'
    }
  | {
      type: 'Proxy'
      config: MavenProxyConfigType
    }
