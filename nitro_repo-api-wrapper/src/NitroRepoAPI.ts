import axios, { AxiosInstance, AxiosResponse } from "axios";
import { Result } from "ts-results";

export let apiURL: string;

export let apiClient: AxiosInstance = axios.create({
    headers: {
        "Content-Type": "application/json",
        Accept: "application/json",
    },
});

export function init(myURL: string) {
    apiURL = myURL
    apiClient = axios.create({
        baseURL: myURL,
        withCredentials: true,
        headers: {
            "Access-Control-Allow-Credentials": true,
            "Content-Type": "application/json",
            Accept: "application/json",
        },
    });
}
export function headers(token: string | undefined) {
    if (token == undefined) {
        return {}
    } else {
        return {
            headers: {
                Authorization: "Bearer " + token,
            },
        }
    }
}
export interface APIError {
    user_friendly_message: string;
    code: number;
}

export const INVALID_LOGIN: APIError = {
    user_friendly_message: "Invalid Username or Password",
    code: 401,
};
export const NOT_AUTHORIZED: APIError = {
    user_friendly_message: "Not Authorized for that action",
    code: 401,
};
export const INTERNAL_ERROR: APIError = {
    user_friendly_message: "Internal Error Occured ",
    code: 500,
};

export function createAPIError(code: number, message: string): APIError {
    let value: APIError = {
        user_friendly_message: message,
        code: code,
    };
    return value;
}


export interface BasicResponse<T> {
    success: boolean;
    data: T;
    status_code: number;
}
