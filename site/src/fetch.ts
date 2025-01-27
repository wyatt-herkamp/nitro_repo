import { apiURL } from "./config";
import { sessionStore } from "./stores/session";

export interface FetchOptions {
  headers?: [string, string][];
}
export function fetchGetRequest(path: string, options?: FetchOptions): Promise<Response> {
  const request = new Request(makeApiUrl(path), {
    method: "GET",
  });
  if (options === undefined) {
    options = {};
  }
  for (const header of options.headers ?? []) {
    request.headers.set(header[0], header[1]);
  }

  const store = sessionStore();
  if (store.session !== undefined) {
    request.headers.set("Authorization", `Session ${store.session.session_id}`);
  }

  return fetch(request);
}

function makeApiUrl(path: string): string {
  let apiUrl: string = apiURL;
  if (apiUrl.endsWith("/")) {
    apiUrl = apiUrl.substring(0, apiUrl.length - 1);
  }

  if (path.startsWith("/")) {
    path = path.substring(1);
  }

  return `${apiUrl}/${path}`;
}
