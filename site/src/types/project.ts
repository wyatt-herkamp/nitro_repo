export interface RawProject {
  id: string;
  scope?: string;
  name: string;
  project_key: string;
  latest_release?: string;
  latest_pre_release?: string;
  repository_id: string;
  storage_path: string;
  updated_at: string;
  created_at: string;
}

export interface RawProjectVersion {
  id: number;
  project_id: string;
  version: string;
  release_type: string;
}

export class Project {
  id: string;
  scope?: string;
  name: string;
  latest_release?: string;
  latest_pre_release?: string;
  project_key: string;
  repository_id: string;
  storage_path: string;
  updated_at: Date;
  created_at: Date;

  constructor(data: RawProject) {
    this.id = data.id;
    this.scope = data.scope;
    this.name = data.name;
    this.latest_release = data.latest_release;
    this.latest_pre_release = data.latest_pre_release;
    this.project_key = data.project_key;
    this.storage_path = data.storage_path;
    this.repository_id = data.repository_id;
    this.updated_at = new Date(data.updated_at);
    this.created_at = new Date(data.created_at);
  }
}
export class ProjectVersion {
  id: number;
  project_id: string;
  version: string;
  release_type: string;
  constructor(data: RawProjectVersion) {
    this.id = data.id;
    this.project_id = data.project_id;
    this.version = data.version;
    this.release_type = data.release_type;
  }
}
