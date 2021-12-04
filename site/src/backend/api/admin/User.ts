import { BasicResponse, DEFAULT_USER_LIST, User, UserList } from "../../Response";
import http from "@/http-common";
import { Err, Ok, Result } from "ts-results";
import { APIError, INTERNAL_ERROR, INVALID_LOGIN, NOT_AUTHORIZED } from "../../NitroRepoAPI";
export async function createNewUser(name: string, username: string, password: string,email: string, token: string) {

    let newUser = {
        name: name,
        username: username,
        email: email,
        password: password,
        permissions: { deployer: false, admin: false },
    };
    return await http.post("/api/admin/user/add", newUser, {
        headers: {
            Authorization: "Bearer " + token,
        }
    })
        .then((result) => {
            const resultData = result.data;
            let value = JSON.stringify(resultData);

            let response: BasicResponse<unknown> = JSON.parse(value);

            if (response.success) {
                return Ok(response.data as User);
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