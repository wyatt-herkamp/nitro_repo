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
}export interface RepoSettings {



}export interface Repository {

    id: number;
    name: string;
    repo_type: string;
    settings: RepoSettings;
    storage: number;
    created: number;

}export interface RepositoryList {

    repositories: Array<Repository>
}