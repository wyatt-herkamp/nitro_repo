import axios, {AxiosInstance} from "axios";

export const apiURL = import.meta.env.VITE_API_URL;

const apiClient: AxiosInstance = axios.create({
    baseURL: apiURL,
    headers: {
        "Content-Type": "application/json",
        Accept: "application/json",
    },
});

export default apiClient;
