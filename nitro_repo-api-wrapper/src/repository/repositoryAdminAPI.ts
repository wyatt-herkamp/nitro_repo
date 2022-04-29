import { Err, Ok } from "ts-results";
import { createAPIError, INTERNAL_ERROR, NOT_AUTHORIZED, apiClient, BasicResponse, headers } from "../NitroRepoAPI";
import { Repository, RepositoryList } from "./repositoryTypes";

export async function createNewRepository(
    name: string,
    storage: string,
    type: string,
    token: string | undefined
) {
    return apiClient
        .post(
            "/api/admin/repository/add",
            { name: name, storage: storage, repo_type: type },
            headers(token)
        )
        .then(
            (result) => {
                const resultData = result.data;
                let value = JSON.stringify(resultData);

                let response: BasicResponse<unknown> = JSON.parse(value);

                if (response.success) {
                    return Ok(response.data as Repository);
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
                            createAPIError(409, "A Repository by that name already exists")
                        );
                    } else if ((err.response.status == 404)) {
                        return Err(
                            createAPIError(404, "Unable to find a Storage by that name")
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

export async function deleteRepository(
    name: string,
    storage: string,
    deleteFiles: boolean,
    token: string | undefined
) {
    return apiClient
        .delete(
            "/api/admin/repository/" + storage + "/" + name + "?delete_files=" + deleteFiles,
            headers(token)
        )
        .then(
            (result) => {
                const resultData = result.data;
                let value = JSON.stringify(resultData);

                let response: BasicResponse<unknown> = JSON.parse(value);

                if (response.success) {
                    return Ok(response.data as boolean);
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
                            createAPIError(409, "A Repository by that name already exists")
                        );
                    } else if ((err.response.status == 404)) {
                        return Err(
                            createAPIError(404, "Unable to find a Storage by that name")
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

export async function setActiveStatus(
    storage: String,
    repository: String,
    active: boolean,
    token: string | undefined
) {
    return apiClient
        .patch(
            "/api/admin/repository/" + storage + "/" + repository + "/active/" + active,
            {},
            headers(token)
        )
        .then(
            (result) => {
                const resultData = result.data;
                let value = JSON.stringify(resultData);

                let response: BasicResponse<unknown> = JSON.parse(value);

                if (response.success) {
                    return Ok(response.data as Repository);
                } else {
                    return Err(INTERNAL_ERROR);
                }
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

export async function setPolicy(storage: String,
    repository: String, policy: string, token: string | undefined) {
    return apiClient
        .patch(
            "/api/admin/repository/" + storage + "/" + repository + "/policy/" + policy,
            {},
            headers(token)
        )
        .then(
            (result) => {
                const resultData = result.data;
                let value = JSON.stringify(resultData);

                let response: BasicResponse<unknown> = JSON.parse(value);

                if (response.success) {
                    return Ok(response.data as Repository);
                } else {
                    return Err(INTERNAL_ERROR);
                }
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

export async function updateBadge(
    storage: String,
    repository: String,
    badgeStyle: string,
    labelColor: string,
    color: string,
    token: string | undefined
) {
    return apiClient
        .patch(
            "/api/admin/repository/" + storage + "/" + repository + "/modify/settings/badge",
            { style: badgeStyle, label_color: labelColor, color: color },
            headers(token)
        )
        .then(
            (result) => {
                const resultData = result.data;
                let value = JSON.stringify(resultData);

                let response: BasicResponse<unknown> = JSON.parse(value);

                if (response.success) {
                    return Ok(response.data as Repository);
                } else {
                    return Err(INTERNAL_ERROR);
                }
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

export async function updateFrontend(
    storage: String,
    repository: String,
    enabled: boolean,
    pageProvider: string,
    token: string | undefined
) {
    // Manually converting data to JSON because JSON.stringify is convering booleans to strings?
    return apiClient
        .patch(
            "/api/admin/repository/" + storage + "/" + repository + "/modify/settings/frontend",
            `{"page_provider":"${pageProvider}","enabled":${enabled}}`,
            headers(token)
        )
        .then(
            (result) => {
                const resultData = result.data;
                let value = JSON.stringify(resultData);

                let response: BasicResponse<unknown> = JSON.parse(value);

                if (response.success) {
                    return Ok(response.data as Repository);
                } else {
                    return Err(INTERNAL_ERROR);
                }
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

export async function setVisibility(
    storage: String,
    repository: String,
    visibility: string,
    token: string | undefined
) {
    return apiClient
        .patch(
            "/api/admin/repository/"
            + storage + "/" + repository +
            "/modify/security/visibility/" +
            visibility,
            {},
            headers(token)
        )
        .then(
            (result) => {
                const resultData = result.data;
                let value = JSON.stringify(resultData);

                let response: BasicResponse<unknown> = JSON.parse(value);

                if (response.success) {
                    return Ok(response.data as Repository);
                } else {
                    return Err(INTERNAL_ERROR);
                }
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

export async function clearAll(storage: String,
    repository: String, what: string, token: string | undefined) {
    return apiClient
        .patch(
            "/api/admin/repository/" + storage + "/" + repository + "/clear/security/" + what,
            {},
            headers(token)
        )
        .then(
            (result) => {
                const resultData = result.data;
                let value = JSON.stringify(resultData);

                let response: BasicResponse<unknown> = JSON.parse(value);

                if (response.success) {
                    return Ok(response.data as Repository);
                } else {
                    return Err(INTERNAL_ERROR);
                }
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

export async function addOrRemoveReadersOrDeployers(
    storage: String,
    repository: String,
    what: string,
    action: string,
    user: number,
    token: string | undefined
) {
    return apiClient
        .patch(
            "/api/admin/repository/"
            + storage + "/" + repository +
            "/modify/security/" +
            what +
            "/" +
            action +
            "/" +
            user,
            {},
            headers(token)
        )
        .then(
            (result) => {
                const resultData = result.data;
                let value = JSON.stringify(resultData);

                let response: BasicResponse<unknown> = JSON.parse(value);

                if (response.success) {
                    return Ok(response.data as Repository);
                } else {
                    return Err(INTERNAL_ERROR);
                }
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

export async function updateDeployReport(
    storage: String,
    repository: String,
    active: boolean,
    values: Array<string>,
    token: string | undefined
) {
    return apiClient
        .patch(
            "/api/admin/repository/" + storage + "/" + repository + "/modify/deploy/report",
            { active: active, values: values },
            headers(token)
        )
        .then(
            (result) => {
                const resultData = result.data;
                let value = JSON.stringify(resultData);

                let response: BasicResponse<unknown> = JSON.parse(value);

                if (response.success) {
                    return Ok(response.data as Repository);
                } else {
                    return Err(INTERNAL_ERROR);
                }
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

export async function updateOrAddWebhppl(
    storage: String,
    repository: String,
    name: string,
    handler: string,
    settings: Map<string, any>,
    token: string | undefined
) {
    return apiClient
        .put(
            "/api/admin/repository/" + storage + "/" + repository + "/modify/deploy/webhook/add",
            { id: name, handler: handler, settings: settings },
            headers(token)
        )
        .then(
            (result) => {
                const resultData = result.data;
                let value = JSON.stringify(resultData);

                let response: BasicResponse<unknown> = JSON.parse(value);

                if (response.success) {
                    return Ok(response.data as Repository);
                } else {
                    return Err(INTERNAL_ERROR);
                }
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

export async function deleteWebhook(storage: String,
    repository: String, name: string, token: string | undefined) {
    return apiClient
        .delete("/api/admin/repository/" + storage + "/" + repository + "/modify/deploy/webhook/" + name, headers(token))
        .then(
            (result) => {
                const resultData = result.data;
                let value = JSON.stringify(resultData);

                let response: BasicResponse<unknown> = JSON.parse(value);

                if (response.success) {
                    return Ok(response.data as Repository);
                } else {
                    return Err(INTERNAL_ERROR);
                }
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

export async function getRepositories(token: string | undefined): Promise<RepositoryList | undefined> {
    const value = await apiClient.get("/api/admin/repositories/list", headers(token));
    if (value.status != 200) {
        return undefined;
    }
    const data = value.data as BasicResponse<unknown>;
    if (data.success) {
        return data.data as RepositoryList;
    }

    return undefined;
}

export async function getRepositoriesByStorage(token: string | undefined, storage: string): Promise<RepositoryList | undefined> {
    const value = await apiClient.get("/api/admin/repositories/" + storage + "/list", headers(token));

    if (value.status != 200) {
        return undefined;
    }
    const data = value.data as BasicResponse<unknown>;
    if (data.success) {
        return data.data as RepositoryList;
    }

    return undefined;
}


export async function getRepoByNameAndStorage(
    token: string | undefined,
    storage: string,
    repo: string
): Promise<Repository | undefined> {
    const url = "/api/admin/repositories/get/" + storage + "/" + repo;
    const value = await apiClient.get(
        url, headers(token)
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