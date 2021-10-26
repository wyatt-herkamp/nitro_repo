import axios, { AxiosInstance } from "axios";
export const baseURL = "http://127.0.0.1:6742";
const apiClient: AxiosInstance = axios.create({
  baseURL: baseURL,
  headers: {
    "Content-Type": "application/json",
    Accept: "application/json",
  },
});

export default apiClient;
