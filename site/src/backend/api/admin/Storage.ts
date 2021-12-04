import { BasicResponse, Repository, User } from "../../Response";
import http from "@/http-common";
import { Err, Ok, Result } from "ts-results";
import { APIError, createAPIError, INTERNAL_ERROR, INVALID_LOGIN, NOT_AUTHORIZED } from "../../NitroRepoAPI";
import { AuthToken } from "../User";
import { Storage } from "@/backend/Response";
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
