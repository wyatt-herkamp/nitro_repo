import axios from "axios";
import router from "@/router";

export const apiURL =
  import.meta.env.VITE_API_URL === undefined
    ? document.baseURI
    : (import.meta.env.VITE_API_URL as string);
export function makeURL(endpoint: string) {
  let url = apiURL;
  if (apiURL.endsWith("/")) {
    url = url.substring(0, url.length - 1);
  }
  if (endpoint.startsWith("/")) {
    return `${url}${endpoint}`;
  } else {
    return `${url}/${endpoint}`;
  }
}
console.log(`apiURL: ${apiURL}`);

const apiClient = axios.create({
  baseURL: apiURL,
  headers: {
    "Content-Type": "application/json",
    Accept: "application/json",
  },
  withCredentials: true,
});

export default { apiClient, apiURL };
