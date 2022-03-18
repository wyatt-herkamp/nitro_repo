export interface BasicResponse<T> {
  success: boolean;
  data: T;
  status_code: number;
}

export interface Storage {
  name: string;
  public_name: string;
  created: number;
}
export const DEFAULT_STORAGE: Storage = {
  name: "",
  public_name: "",
  created: 0,
};
export interface StorageList {
  storages: Array<Storage>;
}
export const DEFAULT_STORAGE_LIST: StorageList = {
  storages: [],
};
export interface SecurityRules {
  visibility: string;
  readers: Array<number>;
  deployers: Array<number>;
}
export const DEFAULT_SECURITY: SecurityRules = {
  visibility: "",
  readers: [],
  deployers: [],
};
export interface Frontend {
  page_provider: string;
  enabled: boolean;
}
export const DEFAULT_FRONTEND: Frontend = {
  page_provider: "",
  enabled: false,
};
export interface BadgeSettings {
  style: string;
  label_color: string;
  color: string;
}
export const DEFAULT_BADGE: BadgeSettings = {
  style: "",
  label_color: "",
  color: "",
};
export interface RepoSettings {
  policy: string;
  active: boolean;
  re_deployment: boolean;
  frontend: Frontend;
  badge: BadgeSettings;
}
export const DEFAULT_REPO_SETTINGS: RepoSettings = {
  policy: "",
  active: false,
  re_deployment: false,
  frontend: DEFAULT_FRONTEND,
  badge: DEFAULT_BADGE,
};

export interface DeploySettings {
  report_generation: ReportGeneration;
  webhooks: Array<Webhook>;
}
export const DEFAULT_DEPLOY_SETTINGS: DeploySettings = {
  report_generation: { active: true, values: [] },
  webhooks: [],
};

export interface ReportGeneration {
  active: boolean;
  values: Array<string>;
}
export interface Webhook {
  id: string;
  handler: string;
  settings: Map<String, any>;
}
export interface Repository {
  id: number;
  name: string;
  repo_type: string;
  settings: RepoSettings;
  deploy_settings: DeploySettings;
  security: SecurityRules;
  storage: string;
  created: number;
}
export const DEFAULT_REPO: Repository = {
  id: 0,
  name: "",
  repo_type: "",
  settings: DEFAULT_REPO_SETTINGS,
  security: DEFAULT_SECURITY,
  storage: "",
  created: 0,
  deploy_settings: DEFAULT_DEPLOY_SETTINGS,
};
export interface User {
  id: number;
  name: string;
  username: string;
  email: string;
  permissions: UserPermissions;
  created: number;
}

export interface UserListResponse {
  id: number;
  name: string;
}

export interface UserPermissions {
  admin: boolean;
  deployer: boolean;
}

export interface RepositoryListResponse {
  name: string;
  repo_type: string;
  storage: string;
}
export interface RepositoryList {
  repositories: Array<RepositoryListResponse>;
}
export const DEFAULT_REPO_LIST: RepositoryList = {
  repositories: [],
};
export interface UserList {
  users: Array<User>;
}
export const DEFAULT_USER_LIST: UserList = {
  users: [],
};
export interface SettingReport {
  email: EmailSettings;
  general: GeneralSetting;
}

export interface EmailSettings {
  email_username: DBSetting;
  email_host: DBSetting;
  encryption: DBSetting;
  from: DBSetting;
  port: DBSetting;
}

export interface DBSetting {
  id: number;
  setting: Setting;
  value: string;
  updated: number;
}

export interface Setting {
  key: string;
  name: string;
  default: null | string;
  optional: null;
  properties: null;
  options: string[] | null;
  public: boolean | null;
}

export interface GeneralSetting {
  name: DBSetting;
  installed: DBSetting;
  version: DBSetting;
}

export interface FileResponse {
  name: string;
  full_path: string;
  directory: boolean;
  data: Map<string, any>;
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
  frontend_response: null;
}

export interface ProjectData {
  name:        string;
  description: string;
  source:      null;
  licence:     null;
  versions:    Versions;
  created:     number;
}

export interface Versions {
  latest_version: string;
  latest_release: string;
  versions:       Version[];
}

export interface Version {
  version:  string;
  time:     number;
  snapshot: boolean;
}

enum Policy {
  Release,
  Snapshot,
  Mixed,
}
