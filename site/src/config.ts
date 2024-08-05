export const apiURL =
  import.meta.env.VITE_API_URL === undefined
    ? document.baseURI
    : (import.meta.env.VITE_API_URL as string)
