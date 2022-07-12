import { User, UserList, UserPermissions } from "./userTypes";
import { apiClient, BasicResponse, createAPIError, INTERNAL_ERROR, NOT_AUTHORIZED } from "../NitroRepoAPI";
import { Err, Ok } from "ts-results";



/**
 * @deprecated The nitro_repo_api-wrapper is deprecated and will be removed in a future release.
 */
export async function createNewUser(
    name: string,
    username: string,
    password: string,
    email: string,
    token: string
) {
    let newUser = {
        name: name,
        username: username,
        email: email,
        password: password,
        permissions: { deployer: false, admin: false },
    };
    return apiClient
        .post("/api/admin/user/add", newUser, {
            headers: {
                Authorization: "Bearer " + token,
            },
        })
        .then(
            (result) => {
                const resultData = result.data;
                let value = JSON.stringify(resultData);

                let response: BasicResponse<unknown> = JSON.parse(value);

                if (response.success) {
                    return Ok(response.data as User);
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
                            createAPIError(
                                409,
                                "A user with that 'user or email' already exists"
                            )
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
/**
 * @deprecated The nitro_repo_api-wrapper is deprecated and will be removed in a future release.
 */
export async function updateOtherPassword(
    user: string,
    password: string,
    token: string
) {
    return apiClient
        .post(
            "/api/admin/user/" + user + "/password",
            { password: password },
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
                    return Ok(response.data as User);
                } else {
                    return Err(INTERNAL_ERROR);
                }
            },
            (err) => {
                if (err.response) {
                    if ((err.response.status == 401)) {
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
/**
 * @deprecated The nitro_repo_api-wrapper is deprecated and will be removed in a future release.
 */
export async function updateNameAndEmail(
    user: string,
    name: string,
    email: string,
    token: string
) {
    return apiClient
        .patch(
            "/api/admin/user/" + user + "/modify",
            { name: name, email: email },
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
                    return Ok(response.data as User);
                } else {
                    return Err(INTERNAL_ERROR);
                }
            },
            (err) => {
                if (err.response) {
                    if ((err.response.status == 401)) {
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
/**
 * @deprecated The nitro_repo_api-wrapper is deprecated and will be removed in a future release.
 */
export async function updatePermission(
    user: string,
    permissions: UserPermissions,
    token: string
) {
    return apiClient
        .patch(
            `/api/admin/user/${user}/modify/permissions`,
            JSON.stringify(permissions),
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
                    return Ok(response.data as User);
                } else {
                    return Err(INTERNAL_ERROR);
                }
            },
            (err) => {
                if (err.response) {
                    if ((err.response.status == 401)) {
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

/**
 * @deprecated The nitro_repo_api-wrapper is deprecated and will be removed in a future release.
 */
export async function getUsers(token: string): Promise<UserList | undefined> {
    //${API_URL}
    const value = await apiClient.get("/api/admin/user/list", {
        headers: {
            Authorization: "Bearer " + token,
        },
    });

    if (value.status != 200) {
        return undefined;
    }
    const data = value.data as BasicResponse<unknown>;
    if (data.success) {
        return data.data as UserList;
    }

    return undefined;
}
/**
 * @deprecated The nitro_repo_api-wrapper is deprecated and will be removed in a future release.
 */
export async function getUserByID(
    token: string,
    id: number
): Promise<User | undefined> {
    //${API_URL}
    const value = await apiClient.get("/api/admin/user/get/" + id, {
        headers: {
            Authorization: "Bearer " + token,
        },
    });

    if (value.status != 200) {
        return undefined;
    }
    const data = value.data as BasicResponse<unknown>;
    if (data.success) {
        return data.data as User;
    }

    return undefined;
}
