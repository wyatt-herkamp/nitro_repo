import { NpmIcon } from 'vue3-simple-icons'
import NPMProjectHelper from './NPMProjectHelper.vue'
export const MavenFrontendDefinition = {
  name: 'npm',
  properName: 'npm',
  projectComponent: {
    component: NPMProjectHelper,
    props: {}
  },
  icons: [
    {
      name: 'NPM',
      component: NpmIcon,
      url: 'https://www.npmjs.com/',
      props: {}
    }
  ]
}
export interface MavenProxyRoute {
  url: string
  name?: string
}
export interface NPMProxyConfigType {
  routes: MavenProxyRoute[]
}
export function defaultProxy(): NPMProxyConfigType {
  return {
    routes: []
  }
}
export type NPMConfigType =
  | {
      type: 'Hosted'
    }
  | {
      type: 'Proxy'
      config: NPMProxyConfigType
    }
