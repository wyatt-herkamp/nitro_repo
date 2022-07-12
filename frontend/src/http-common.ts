import axios from "axios";

export const apiURL =
  import.meta.env.VITE_API_URL === undefined
    ? document.baseURI
    : (import.meta.env.VITE_API_URL as string);

const apiClient = axios.create({
  baseURL: apiURL,
  headers: {
    "Content-Type": "application/json",
    Accept: "application/json",
  },
  withCredentials: true,
});

export default { apiClient, apiURL };
