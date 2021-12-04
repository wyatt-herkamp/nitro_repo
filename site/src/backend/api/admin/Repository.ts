import { BasicResponse, User } from "../../Response";
import http from "@/http-common";
import { Err, Ok, Result } from "ts-results";
import { APIError, createAPIError, INTERNAL_ERROR, INVALID_LOGIN, NOT_AUTHORIZED } from "../../NitroRepoAPI";
import { AuthToken } from "../User";
import { Repository } from "@/backend/Response";
export async function createNewRepository(name: string, storage: string, type: string, token: string) {
    return await http.post("/api/admin/repository/add", { name: name, storage: storage, repo: type }, {
        headers: {
            Authorization: "Bearer " + token,
        }
    })
        .then((result) => {
            const resultData = result.data;
            let value = JSON.stringify(resultData);

            let response: BasicResponse<unknown> = JSON.parse(value);

            if (response.success) {
                return Ok(response.data as Repository);
            } else {
                return Err(INTERNAL_ERROR);
            }
        }, (err) => {
            if (err.response) {
                if (err.response.status = 401) {
                    return Err(NOT_AUTHORIZED);
                } else if (err.response.status = 409) {
                    return Err(createAPIError(409, "A Repository by that name already exists"));
                } else if (err.response.status = 404) {
                    return Err(createAPIError(404, "Unable to find a Storage by that name"));
                }
                return Err(INTERNAL_ERROR);
            } else if (err.request) {
                return Err(INTERNAL_ERROR);
            } else {
                return Err(INTERNAL_ERROR);
            }
        });

} 
export async function setActiveStatus(id: number, active: boolean, token: string) {
    return await http.patch("/api/admin/repository/" + id + "/active/" + active, {}, {
        headers: {
            Authorization: "Bearer " + token,
        }
    })
        .then((result) => {
            const resultData = result.data;
            let value = JSON.stringify(resultData);

            let response: BasicResponse<unknown> = JSON.parse(value);

            if (response.success) {
                return Ok(response.data as Repository);
            } else {
                return Err(INTERNAL_ERROR);
            }
        }, (err) => {
            if (err.response) {
                if (err.response.status = 401) {
                    return Err(NOT_AUTHORIZED);
                }
                return Err(INTERNAL_ERROR);
            } else if (err.request) {
                return Err(INTERNAL_ERROR);
            } else {
                return Err(INTERNAL_ERROR);
            }
        });

} export async function setPolicy(id: number, policy: string, token: string) {
    return await http.patch("/api/admin/repository/" + id + "/policy/" + policy, {}, {
        headers: {
            Authorization: "Bearer " + token,
        }
    })
        .then((result) => {
            const resultData = result.data;
            let value = JSON.stringify(resultData);

            let response: BasicResponse<unknown> = JSON.parse(value);

            if (response.success) {
                return Ok(response.data as Repository);
            } else {
                return Err(INTERNAL_ERROR);
            }
        }, (err) => {
            if (err.response) {
                if (err.response.status = 401) {
                    return Err(NOT_AUTHORIZED);
                }
                return Err(INTERNAL_ERROR);
            } else if (err.request) {
                return Err(INTERNAL_ERROR);
            } else {
                return Err(INTERNAL_ERROR);
            }
        });

} export async function updateBadge(id: number, badgeStyle: string, labelColor: string, color: string, token: string) {
    return await http.patch("/api/admin/repository/" + id + "/modify/settings/badge", { style: badgeStyle, label_color: labelColor, color: color }, {
        headers: {
            Authorization: "Bearer " + token,
        }
    })
        .then((result) => {
            const resultData = result.data;
            let value = JSON.stringify(resultData);

            let response: BasicResponse<unknown> = JSON.parse(value);

            if (response.success) {
                return Ok(response.data as Repository);
            } else {
                return Err(INTERNAL_ERROR);
            }
        }, (err) => {
            if (err.response) {
                if (err.response.status = 401) {
                    return Err(NOT_AUTHORIZED);
                }
                return Err(INTERNAL_ERROR);
            } else if (err.request) {
                return Err(INTERNAL_ERROR);
            } else {
                return Err(INTERNAL_ERROR);
            }
        });

} export async function updateFrontend(id: number, enabled: boolean, pageProvider: string, token: string) {
    return await http.patch("/api/admin/repository/" + id + "/modify/settings/frontend", { enabled: enabled, page_provider: pageProvider }, {
        headers: {
            Authorization: "Bearer " + token,
        }
    })
        .then((result) => {
            const resultData = result.data;
            let value = JSON.stringify(resultData);

            let response: BasicResponse<unknown> = JSON.parse(value);

            if (response.success) {
                return Ok(response.data as Repository);
            } else {
                return Err(INTERNAL_ERROR);
            }
        }, (err) => {
            if (err.response) {
                if (err.response.status = 401) {
                    return Err(NOT_AUTHORIZED);
                }
                return Err(INTERNAL_ERROR);
            } else if (err.request) {
                return Err(INTERNAL_ERROR);
            } else {
                return Err(INTERNAL_ERROR);
            }
        });

}

export async function setVisibility(id: number, visibility: string, token: string) {
    return await http.patch("/api/admin/repository/" + id + "/modify/security/visibility/" + visibility, {}, {
        headers: {
            Authorization: "Bearer " + token,
        }
    })
        .then((result) => {
            const resultData = result.data;
            let value = JSON.stringify(resultData);

            let response: BasicResponse<unknown> = JSON.parse(value);

            if (response.success) {
                return Ok(response.data as Repository);
            } else {
                return Err(INTERNAL_ERROR);
            }
        }, (err) => {
            if (err.response) {
                if (err.response.status = 401) {
                    return Err(NOT_AUTHORIZED);
                }
                return Err(INTERNAL_ERROR);
            } else if (err.request) {
                return Err(INTERNAL_ERROR);
            } else {
                return Err(INTERNAL_ERROR);
            }
        });

} export async function clearAll(id: number, what: string, token: string) {
    return await http.patch("/api/admin/repository/" + id + "/clear/security/" + what, {}, {
        headers: {
            Authorization: "Bearer " + token,
        }
    })
        .then((result) => {
            const resultData = result.data;
            let value = JSON.stringify(resultData);

            let response: BasicResponse<unknown> = JSON.parse(value);

            if (response.success) {
                return Ok(response.data as Repository);
            } else {
                return Err(INTERNAL_ERROR);
            }
        }, (err) => {
            if (err.response) {
                if (err.response.status = 401) {
                    return Err(NOT_AUTHORIZED);
                }
                return Err(INTERNAL_ERROR);
            } else if (err.request) {
                return Err(INTERNAL_ERROR);
            } else {
                return Err(INTERNAL_ERROR);
            }
        });

} export async function addOrRemoveReadersOrDeployers(id: number, what: string, action: string, user: number, token: string) {
    return await http.patch("/api/admin/repository/" + id + "/modify/security/" + what + "/" + action + "/" + user, {}, {
        headers: {
            Authorization: "Bearer " + token,
        }
    })
        .then((result) => {
            const resultData = result.data;
            let value = JSON.stringify(resultData);

            let response: BasicResponse<unknown> = JSON.parse(value);

            if (response.success) {
                return Ok(response.data as Repository);
            } else {
                return Err(INTERNAL_ERROR);
            }
        }, (err) => {
            if (err.response) {
                if (err.response.status = 401) {
                    return Err(NOT_AUTHORIZED);
                }
                return Err(INTERNAL_ERROR);
            } else if (err.request) {
                return Err(INTERNAL_ERROR);
            } else {
                return Err(INTERNAL_ERROR);
            }
        });

} 