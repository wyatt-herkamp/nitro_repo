export interface BasicResponse<T> {
    success: boolean;
    data: T;
    status_code: number;
}
export interface APIError {
    user_friendly_message: String;
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

}export interface StorageList {

    storages: Array<Storage>
}export interface SecurityRules {
}export interface RepoSettings {
    policy: string,
    active: boolean,
    re_deployment: boolean,
    security_rules: SecurityRules

}export interface Repository {

    id: number;
    name: string;
    repo_type: string;
    settings: RepoSettings;
    storage: number;
    created: number;

}export interface User {

    id: number;
    name: string;
    username: string;
    email: string;
    UserPermissions: RepoSettings;
    created: number;

}export interface UserPermissions {
    admin: boolean;
    deployer: boolean


}
export interface RepositoryList {

    repositories: Array<Repository>
}
export interface UserList {

    users: Array<User>
}
