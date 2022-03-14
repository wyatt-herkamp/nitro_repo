import {BasicResponse, Storage} from "../../Response";
import http from "@/http-common";
import {Err, Ok} from "ts-results";
import {createAPIError, INTERNAL_ERROR, NOT_AUTHORIZED,} from "../../NitroRepoAPI";

export async function createNewStorage(
  name: string,
  public_name: string,
  token: string
) {
  return http
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
}
export async function deleteStorage(
  id: number,
  token: string
) {
  return http
    .delete(
      "/api/admin/storages/"+id,
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
