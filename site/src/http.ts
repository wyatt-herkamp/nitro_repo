import axios from 'axios'
import { apiURL } from '@/config'
import { sessionStore } from '@/stores/session'

const axiosInstance = axios.create({
  baseURL: apiURL,
  headers: {
    'Content-Type': 'application/json',
    Accept: 'application/json'
  }
})
axiosInstance.interceptors.request.use((config) => {
  const store = sessionStore()
  if (store.session !== undefined) {
    config.headers.Authorization = `session ${store.session.session_id}`
  }
  return config
})
export default axiosInstance
