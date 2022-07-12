import {apiClient, BasicResponse, createAPIError, INTERNAL_ERROR, NOT_AUTHORIZED} from "../NitroRepoAPI";
import {Storage, StorageList} from "../storage/storageTypes";
import {Err, Ok} from "ts-results";
import { BrowseResponse } from "src/repository/repositoryTypes";
/**
 * @deprecated The nitro_repo_api-wrapper is deprecated and will be removed in a future release.
 */
export async function getStorages(token: string): Promise<Array<Storage> | undefined> {
    const value = await apiClient.get("/api/storages/list", {
        headers: {
            Authorization: "Bearer " + token,
        },
    });

    if (value.status != 200) {
        return undefined;
    }
    const data = value.data as BasicResponse<unknown>;
    if (data.success) {
        return data.data as Array<Storage>;
    }

    return undefined;
}
/**
 * @deprecated The nitro_repo_api-wrapper is deprecated and will be removed in a future release.
 */
export async function getStoragesPublicAccess():  Promise<BrowseResponse | undefined> {
    const value = await apiClient.get("/storages.json", {});

    if (value.status != 200) {
        return undefined;
    }
    const data = value.data as BasicResponse<unknown>;
    if (data.success) {
        return data.data as BrowseResponse;
    }

    return undefined;
}
/**
 * @deprecated The nitro_repo_api-wrapper is deprecated and will be removed in a future release.
 */
export async function getStorage(token: string, id: string): Promise<Storage | undefined> {
    const value = await apiClient.get("/api/storages/id/" + id, {
        headers: {
            Authorization: "Bearer " + token,
        },
    });

    if (value.status != 200) {
        return undefined;
    }
    const data = value.data as BasicResponse<unknown>;
    if (data.success) {
        return data.data as Storage;
    }

    return undefined;
}
/**
 * @deprecated The nitro_repo_api-wrapper is deprecated and will be removed in a future release.
 */
export async function createNewStorage(
    name: string,
    public_name: string,
    token: string
) {
    return apiClient
        .post(
            "/api/admin/storages/add",
            { name: name, public_name: public_name },
            {
                headers: {
                    Authorization: "Bearer " + token,
                },
            }
        )
        .then(
            (result) => {
                const resultData = result.data;
                let value = JSON.stringify(resultData);

                let response: BasicResponse<unknown> = JSON.parse(value);

                if (response.success) {
                    return Ok(response.data as Storage);
                } else {
                    return Err(INTERNAL_ERROR);
                }
            },
            (err) => {
                if (err.response) {
                    if ((err.response.status == 401)) {
                        return Err(NOT_AUTHORIZED);
                    } else if ((err.response.status == 409)) {
                        return Err(
                            createAPIError(409, "A Storage by that name already exists")
                        );
                    }
                    return Err(INTERNAL_ERROR);
                } else if (err.request) {
                    return Err(INTERNAL_ERROR);
                } else {
                    return Err(INTERNAL_ERROR);
                }
            }
        );
}/**
 * @deprecated The nitro_repo_api-wrapper is deprecated and will be removed in a future release.
 */
export async function deleteStorage(
    name: string,
    token: string
) {
    return apiClient
        .delete(
            "/api/admin/storages/"+name,
            {
                headers: {
                    Authorization: "Bearer " + token,
                },
            }
        )
        .then(
            (result) => {
                const resultData = result.data;
                let value = JSON.stringify(resultData);

                let response: BasicResponse<unknown> = JSON.parse(value);

                if (response.success) {
                    return Ok(response.data as Storage);
                } else {
                    return Err(INTERNAL_ERROR);
                }
            },
            (err) => {
                if (err.response) {
                    if ((err.response.status == 401)) {
                        return Err(NOT_AUTHORIZED);
                    } else if ((err.response.status == 409)) {
                        return Err(
                            createAPIError(409, "A Storage by that name already exists")
                        );
                    }
                    return Err(INTERNAL_ERROR);
                } else if (err.request) {
                    return Err(INTERNAL_ERROR);
                } else {
                    return Err(INTERNAL_ERROR);
                }
            }
        );
}
