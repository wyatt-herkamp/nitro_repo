import http from "@/http-common";
import {BasicResponse, DEFAULT_REPO_LIST, FileResponse, Repository, RepositoryList,} from "../Response";

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

export async function getRepoByID(
    token: string,
    id: number
): Promise<Repository | undefined> {
    const value = await http.get("/api/repositories/get/" + id, {
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
    token: string,
    storage: string,
    repo: string
): Promise<Repository | undefined> {
    const value = await http.get(
        "/api/repositories/get/" + storage + "/" + repo,
        {
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
