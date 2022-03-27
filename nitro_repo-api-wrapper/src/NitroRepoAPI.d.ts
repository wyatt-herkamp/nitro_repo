import { AxiosInstance } from "axios";
export declare let apiURL: string;
export declare let apiClient: AxiosInstance;
export declare function init(myURL: string): void;
export interface APIError {
    user_friendly_message: string;
    code: number;
}
export declare const INVALID_LOGIN: APIError;
export declare const NOT_AUTHORIZED: APIError;
export declare const INTERNAL_ERROR: APIError;
export declare function createAPIError(code: number, message: string): APIError;
