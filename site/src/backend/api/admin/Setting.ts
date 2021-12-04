import { BasicResponse, DBSetting, Setting, User } from "../../Response";
import http from "@/http-common";
import { Err, Ok, Result } from "ts-results";
import { APIError, createAPIError, INTERNAL_ERROR, INVALID_LOGIN, NOT_AUTHORIZED } from "../../NitroRepoAPI";
import { AuthToken } from "../User";
import { Repository } from "@/backend/Response";


export async function updateSetting(name: string, value: string, token: string) {
    return await http.post("/api/admin/setting/" + name+ "/update", { value: value, }, {
        headers: {
            Authorization: "Bearer " + token,
        }
    })
        .then((result) => {
            const resultData = result.data;
            let value = JSON.stringify(resultData);

            let response: BasicResponse<unknown> = JSON.parse(value);

            if (response.success) {
                return Ok(response.data as DBSetting);
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