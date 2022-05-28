import axios, { AxiosInstance } from "axios";
import { Ok, Err } from "ts-results";
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
    "Content-Type": "application/json",
    Accept: "application/json",
  },
});
apiClient.interceptors.response.use(function (response) {
  return new Ok(response.data);

}, function (error) {
  if (error.response) {
    return Err(error.response)
  } else if (error.request) {
  console.log('Error', error);
  return Err(undefined);
} else {
  // Something happened in setting up the request that triggered an Error
  console.log('Error', error);
  return Err(undefined);
}
})

export default apiClient;
