import http from '@/http'
import MavenConfig from '@/components/admin/repository/configs/MavenConfig.vue'
import PushRulesConfig from '@/components/admin/repository/configs/PushRulesConfig.vue'
import SecurityConfig from '@/components/admin/repository/configs/SecurityConfig.vue'
import type { Component } from 'vue'
import RepositoryPageEditor from '@/components/admin/repository/configs/RepositoryPageEditor.vue'

export interface RepositoryTypeDescription {
  type_name: string
  name: string
  description: string
  documentation_url?: string
  is_stable: boolean
  required_configs: string[]
}
enum Visibility {
  Private = 'Private',
  Public = 'Public',
  Hidden = 'Hidden'
}
export interface RepositoryWithStorageName {
  id: string
  storage_name: string
  storage_id: string
  name: string
  repository_type: string
  active: boolean
  visibility: Visibility
  updated_at: string
  created_at: string
}
export interface ConfigDescription {
  name: string
  description: string
  documentation_url?: string
}
export interface MavenProxyConfigType {
  goTo: string
}
export type MavenConfigType =
  | {
      type: 'Hosted'
    }
  | {
      type: 'Proxy'
      config: MavenProxyConfigType
    }
export interface ConfigType {
  name: string
  title: string
  component: Component
}

export const configTypes: ConfigType[] = [
  {
    name: 'maven',
    title: 'Maven',
    component: MavenConfig
  },
  {
    name: 'page',
    title: 'Page',
    component: RepositoryPageEditor
  }
]

export function getConfigType(name: string): ConfigType | undefined {
  return configTypes.find((configType) => configType.name === name)
}

export async function getConfigTypeDefault(name: string): Promise<any> {
  return await http
    .get<any>(`/api/repository/config/${name}/default`)
    .then((response: any) => {
      return response.data
    })
    .catch((error: any) => {
      console.error(error)
    })
}

export async function validateConfig(name: string, config: any): Promise<any> {
  return await http
    .post<any>(`/api/repository/config/${name}/validate`, config)
    .then((response: any) => {
      return response.data
    })
    .catch((error: any) => {
      console.error(error)
    })
}
export enum PageType {
  Markdown = 'Markdown',
  HTML = 'HTML',
  None = 'None'
}
export interface RepositoryPage {
  page_type: PageType
  content: string | undefined
}
