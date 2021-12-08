import { BasicResponse, Storage, User } from "../../Response";
import http from "@/http-common";
import { Err, Ok, Result } from "ts-results";
import {
  APIError,
  createAPIError,
  INTERNAL_ERROR,
  INVALID_LOGIN,
  NOT_AUTHORIZED,
} from "../../NitroRepoAPI";
import { AuthToken } from "../User";

export async function createNewStorage(
  name: string,
  public_name: string,
  token: string
) {
  return await http
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
          if ((err.response.status = 401)) {
            return Err(NOT_AUTHORIZED);
          } else if ((err.response.status = 409)) {
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
