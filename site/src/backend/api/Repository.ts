import http from "@/http-common";
import { BasicResponse, DEFAULT_REPO_LIST, FileResponse, Project, Repository, RepositoryList, } from "../Response";

export async function getRepositories(token: string) {
  const value = await http.get("/api/admin/repositories/list", {
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

export async function getRepoByID(
  token: string,
  id: number
): Promise<Repository | undefined> {
  const value = await http.get("/api/admin/repositories/get/" + id, {
    headers: {
      Authorization: "Bearer " + token,
    },
  });

  if (value.status != 200) {
    return undefined;
  }
  const data = value.data as BasicResponse<unknown>;
  if (data.success) {
    return data.data as Repository;
  }

  return undefined;
}

export async function getRepoByNameAndStorage(
  token: string | undefined,
  storage: string,
  repo: string
): Promise<Repository | undefined> {
  const url = "/api/deployer/repositories/get/" + storage + "/" + repo;
  const value = token == undefined ? await http.get(url) : await http.get(
    url, {
    headers: {
      Authorization: "Bearer " + token,
    },
  }
  );

  if (value.status != 200) {
    return undefined;
  }
  const data = value.data as BasicResponse<unknown>;
  if (data.success) {
    return data.data as Repository;
  }

  return undefined;

}
export interface PublicRepositoryInfo {
  id: number;
  name: string;
  repo_type: string;
  storage: string;
  description: string;
  active: boolean;
  visibility: string;
  policy: string;
  created: number;
}

export async function getRepoPublic(
  token: string | undefined,
  storage: string,
  repo: string
): Promise<PublicRepositoryInfo | undefined> {
  const url = "/api/repositories/get/" + storage + "/" + repo;
  const value = token == undefined ? await http.get(url) : await http.get(
    url, {
    headers: {
      Authorization: "Bearer " + token,
    },
  }
  );

  if (value.status != 200) {
    return undefined;
  }
  const data = value.data as BasicResponse<unknown>;
  if (data.success) {
    return data.data as PublicRepositoryInfo;
  }

  return undefined;
}
export async function getRepositoriesPublicAccess(storage: string) {
  const url = "/storages/" + storage + ".json";
  console.log(url);
  const value = await http.get(url, {});

  if (value.status != 200) {
    return [];
  }
  const data = value.data as BasicResponse<unknown>;
  if (data.success) {
    return data.data as Array<string>;
  }

  return [];
}

export async function fileListing(storage: string, repo: string, path: string) {
  const url = "/storages/" + storage + "/" + repo + "/" + path;
  console.log(url);
  const value = await http.get(url, {});

  if (value.status != 200) {
    return [];
  }
  const data = value.data as BasicResponse<unknown>;
  if (data.success) {
    return data.data as Array<FileResponse>;
  }

  return [];
}

export async function getProject(
  token: string | undefined,
  storage: string,
  repo: string,
  project: string,
  version: string,

): Promise<Project | undefined> {
  let url = `/api/project/${storage}/${repo}/${project}`;
  if (version != undefined && version !== "") {
    url = url + "/${version}"
  }
  console.log(url);
  const value = (token == undefined) ? await http.get(url) : await http.get(
    url, {
    headers: {
      Authorization: "Bearer " + token,
    },
  }
  );

  if (value.status != 200) {
    return undefined;
  }

  const data = value.data as BasicResponse<unknown>;
  if (data.success) {
    return data.data as Project;
  }

  return undefined;
}
