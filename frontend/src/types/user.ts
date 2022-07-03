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
  disabled: boolean;
  admin: boolean;
  user_manager: boolean;
  repository_manager: boolean;
  deployer?: RepositoryPermissions;
  viewer?: RepositoryPermissions;
}

export interface RepositoryPermissions {
  permissions: Array<string>;
}

export interface UserList {
  users: Array<User>;
}
