import { apiClient, APIError, BasicResponse, createAPIError, headers, INTERNAL_ERROR, NOT_AUTHORIZED } from "../NitroRepoAPI";
import { Storage, StorageList } from "../storage/storageTypes";
import { Err, Ok, Result } from "ts-results";
import { BrowseResponse } from "src/repository/repositoryTypes";

export async function getStorages(token: string | undefined): Promise<Result<Array<Storage>, APIError>> {
    return apiClient
        .get(
            "/api/admin/storages",
            headers(token),
        )
        .then(
            (result) => {
                const resultData = result.data;
                let value = JSON.stringify(resultData);

                let response: Array<Storage> = JSON.parse(value);
                return Ok(response);
            },
            (err) => {
                if (err.response) {
                    if ((err.response.status = 401)) {
                        return Err(NOT_AUTHORIZED);
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

export async function getStoragesPublicAccess(): Promise<BrowseResponse | undefined> {
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

export async function getStorage(token: string | undefined, id: string): Promise<Storage | undefined> {
    const value = await apiClient.get("/api/storages/id/" + id, headers(token));

    if (value.status != 200) {
        return undefined;
    }
    const data = value.data as BasicResponse<unknown>;
    if (data.success) {
        return data.data as Storage;
    }

    return undefined;
}

export async function createNewStorage(
    name: string,
    public_name: string,
    location: string,
    token: string | undefined
) {

    let storageFactory = {
        storage_type: "LocalStorage",
        name: name,
        public_name: public_name,
        handler_config: {
            location: location
        }
    };
    return apiClient
        .post(
            "/storage/new",
            storageFactory,
            headers(token)
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
export async function deleteStorage(
    name: string,
    token: string | undefined
) {
    return apiClient
        .delete(
            "/api/admin/storages/" + name,
            headers(token)
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
