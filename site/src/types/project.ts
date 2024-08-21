import MavenProjectHelper from '@/components/repository/types/maven/MavenProjectHelper.vue'

export interface RawProject {
  id: string
  scope?: string
  name?: string
  project_key?: string
  repository: string
  updated_at: string
  created_at: string
}

export class Project {
  id: string
  scope?: string
  name?: string
  project_key?: string
  repository: string
  updated_at: Date
  created_at: Date
  constructor(data: RawProject) {
    this.id = data.id
    this.scope = data.scope
    this.name = data.name
    this.project_key = data.project_key
    this.repository = data.repository
    this.updated_at = new Date(data.updated_at)
    this.created_at = new Date(data.created_at)
  }
}
