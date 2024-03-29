export interface Repository {
  name: string;
  repository_type: string;
  storage: string;
  visibility: string;
  active: boolean;
  created: number;
  require_token_over_basic: boolean;
}
export interface ReportGeneration {
  active: boolean;
  values: Array<string>;
}

export interface RepositoryListResponse {
  name: string;
  repo_type: string;
  storage: string;
}
export interface RepositoryList {
  repositories: Array<RepositoryListResponse>;
}
export interface SecurityRules {
  visibility: string;
  readers: Array<number>;
  deployers: Array<number>;
}
export interface BadgeSettings {
  style: string;
  label_color: string;
  color: string;
}
export interface RepoSettings {
  policy: string;
  active: boolean;
  re_deployment: boolean;
  frontend: Frontend;
  badge: BadgeSettings;
}
export interface Frontend {
  page_provider: string;
  enabled: boolean;
}

export interface Version {
  version: string;
  artifacts: string[];
}

export interface RepoSummary {
  name: string;
  storage: string;
  page_provider: string;
  repo_type: string;
  visibility: string;
}

export interface Project {
  repo_summary: RepoSummary;
  project: ProjectData;
  version: VersionData;
  frontend_response: null;
}

export interface VersionData {
  name: string;
  version: string;
  description: string;
  source: null;
  licence: null;
  created: number;
}

export interface ProjectData {
  versions: Versions;
  created: number;
}

export interface Versions {
  latest_version: string;
  latest_release: string;
  versions: Version[];
}

export interface Version {
  version: string;
  time: number;
  snapshot: boolean;
}

export enum Policy {
  Release,
  Snapshot,
  Mixed,
}

export interface VersionBrowseResponse {
  Project?: ProjectData;
  Version: string;
}
export interface ResponseType {
  Project?: Project;
  Repository?: Repository;
  Version?: VersionBrowseResponse;
}
export interface BrowseResponse {
  response_type: ResponseType | string;
  files: Array<FileResponse>;
  active_dir: string;
}
export interface FileResponse {
  name: string;
  full_path: string;
  directory: boolean;
  file_size: number;
  response_type: ResponseType | string;
  created: number;
}
