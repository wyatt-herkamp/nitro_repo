import axios, { AxiosInstance } from "axios";

const apiClient: AxiosInstance = axios.create({
    baseURL: "http://127.0.0.1:6742",
});

export default apiClient;
