import http from "@/http-common";
import { BasicResponse, RepositoryList, DEFAULT_REPO_LIST } from "../Response";
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
