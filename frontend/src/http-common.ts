import axios, { AxiosInstance } from "axios";

export let apiURL: string;
if (import.meta.env.VITE_API_URL == undefined) {
  apiURL = appURL;
} else {
  apiURL = import.meta.env.VITE_API_URL;
}

const apiClient: AxiosInstance = axios.create({
  baseURL: apiURL,
  withCredentials: true,

  headers: {
    "Access-Control-Allow-Credentials": true,
    "Content-Type": "application/json",
    Accept: "application/json",
  },
});

export default apiClient;
