export interface BasicResponse<T> {
    success: boolean;
    data: T;
    status_code: number;
}
export interface APIError {
    user_friendly_message: String;
    error_code: String;
}

export interface LoginRequest {
    auth_token: AuthToken;
}
export interface AuthToken {
    id: number;
    user: number;
    token: String;
    expiration: number;
    created: number;
}