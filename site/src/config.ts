export const apiURL =
  import.meta.env.VITE_API_URL === undefined
    ? document.baseURI
    : (import.meta.env.VITE_API_URL as string);

export function websocketUrl(): string {
  return apiURL.replace("http", "ws");
}

export function websocketPath(path: string): string {
  let ws_url = websocketUrl();

  if (ws_url.endsWith("/")) {
    ws_url = ws_url.substring(0, ws_url.length - 1);
  }

  if (path.startsWith("/")) {
    path = path.substring(1);
  }

  return `${ws_url}/${path}`;
}

console.log("API URL: ", apiURL);
console.log("Websocket URL: ", websocketUrl());
console.log("Document base URI: ", document.baseURI);
