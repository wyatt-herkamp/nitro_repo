import { apiClient, BasicResponse } from "../NitroRepoAPI";
import { BrowseResponse, FileResponse, Project } from "./repositoryTypes";

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
    const value = token == undefined ? await apiClient.get(url) : await apiClient.get(
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
export async function getRepositoriesPublicAccess(storage: string): Promise<BrowseResponse | undefined> {
    const url = "/storages/" + storage + ".json";
    const value = await apiClient.get(url, {});

    if (value.status != 200) {
        return undefined;
    }
    const data = value.data as BasicResponse<unknown>;
    if (data.success) {
        return data.data as BrowseResponse;
    }

    return undefined;
}

export async function browse(path: string, token: string | undefined): Promise<BrowseResponse | undefined> {
    const url = "/storages/" + path;
    const value = (token == undefined) ? await apiClient.get(url) : await apiClient.get(
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
        return data.data as BrowseResponse;
    }

    return undefined;
}

export async function fileListing(storage: string, repo: string, path: string): Promise<BrowseResponse | undefined> {
    const url = "/storages/" + storage + "/" + repo + "/" + path;
    const value = await apiClient.get(url, {});

    if (value.status != 200) {
        return undefined;
    }
    const data = value.data as BasicResponse<unknown>;
    if (data.success) {
        return data.data as BrowseResponse;
    }

    return undefined;
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
    const value = (token == undefined) ? await apiClient.get(url) : await apiClient.get(
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