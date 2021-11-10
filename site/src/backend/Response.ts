export interface BasicResponse<T> {
  success: boolean;
  data: T;
  status_code: number;
}
export interface APIError {
  user_friendly_message: string;
  error_code: string;
}

export interface LoginRequest {
  auth_token: AuthToken;
}
export interface AuthToken {
  id: number;
  user: number;
  token: string;
  expiration: number;
  created: number;
}
export interface Storage {
  id: number;
  name: string;
  public_name: string;
  created: number;
}
export const DEFAULT_STORAGE: Storage = {
  id: 0,
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
  open_to_all_deployers: boolean;
  open_to_all_readers: boolean;
  visibility: string;
  readers: Array<number>;
  deployers: Array<number>;
}

export interface Frontend {
  page_provider: string;
  enabled: boolean;
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
export interface Repository {
  id: number;
  name: string;
  repo_type: string;
  settings: RepoSettings;
  security: SecurityRules;
  storage: number;
  created: number;
}
export interface User {
  id: number;
  name: string;
  username: string;
  email: string;
  permissions: UserPermissions;
  created: number;
}export interface UserListResponse {
  id: number;
  name: string;
}
export interface UserPermissions {
  admin: boolean;
  deployer: boolean;
}
export interface RepositoryListResponse {
  id: number;
  name: string;
  repo_type: string;
  storage: number;
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
  name: string,
  full_path: string,
  directory: boolean,
  data: Map<string, any>
}