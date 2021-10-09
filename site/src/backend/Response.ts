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
export interface StorageList {
  storages: Array<Storage>;
}
export const DEFAULT_STORAGE_LIST :StorageList={
  storages: []
}
export interface SecurityRules {
  open_to_all_deployers: boolean;
  open_to_all_readers: boolean;
  visibility: string;
  readers: Array<number>;
  deployers: Array<number>;
}
export interface RepoSettings {
  policy: string;
  active: boolean;
  re_deployment: boolean;
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
}
export interface UserPermissions {
  admin: boolean;
  deployer: boolean;
}
export interface RepositoryList {
  repositories: Array<Repository>;
}
export interface UserList {
  users: Array<User>;
}


export interface SettingReport {
  email:    EmailSettings;
  general:  DBSetting;
}

export interface EmailSettings {
  email_username: DBSetting;
  email_host:     DBSetting;
  encryption:     DBSetting;
  from:           DBSetting;
  port:           DBSetting;
}

export interface DBSetting {
  id:      number;
  setting: Setting;
  value:   string;
  updated: number;
}

export interface Setting {
  key:        string;
  name:       string;
  default:    null | string;
  optional:   null;
  properties: null;
  options:    string[] | null;
  public:     boolean | null;
}

export interface GeneralSetting {
  name: DBSetting;
}
