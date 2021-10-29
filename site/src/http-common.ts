declare const API_URL: string;

import axios, { AxiosInstance } from "axios";
export const baseURL = API_URL;
const apiClient: AxiosInstance = axios.create({
  baseURL: baseURL,
  headers: {
    "Content-Type": "application/json",
    Accept: "application/json",
  },
});

export default apiClient;
