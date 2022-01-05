import {BasicResponse, DBSetting} from "../../Response";
import http from "@/http-common";
import {Err, Ok} from "ts-results";
import {INTERNAL_ERROR, NOT_AUTHORIZED} from "../../NitroRepoAPI";

export async function updateSetting(
  name: string,
  value: string,
  token: string
) {
  return http
    .post(
      "/api/admin/setting/" + name + "/update",
      { value: value },
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
          return Ok(response.data as DBSetting);
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
