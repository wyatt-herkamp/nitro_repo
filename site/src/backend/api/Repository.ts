import http from "@/http-common";
import { BasicResponse, RepositoryList, DEFAULT_REPO_LIST, FileResponse } from "../Response";
export async function getRepositories(token: string) {
  const value = await http.get("/api/repositories/list", {
    headers: {
      Authorization: "Bearer " + token,
    },
  });

  if (value.status != 200) {
    return DEFAULT_REPO_LIST;
  }
  const data = value.data as BasicResponse<unknown>;
  if (data.success) {
    return data.data as RepositoryList;
  }

  return DEFAULT_REPO_LIST;
}
export async function getRepositoriesPublicAccess(storage: string) {
  const url = "/storages/"+storage+".json";
  console.log(url)
  const value = await http.get(url, {
  });

  if (value.status != 200) {
    return [];
  }
  const data = value.data as BasicResponse<unknown>;
  if (data.success) {
    return data.data as Array<string>;
  }

  return [];
}export async function fileListing(storage: string, repo: string, path: string) {
  const url = "/storages/"+storage+"/"+repo+"/"+path;
  console.log(url)
  const value = await http.get(url, {
  });

  if (value.status != 200) {
    return [];
  }
  const data = value.data as BasicResponse<unknown>;
  if (data.success) {
    return data.data as Array<FileResponse>;
  }

  return [];
}