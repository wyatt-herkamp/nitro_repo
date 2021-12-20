import {BasicResponse} from "../Response";
import http from "@/http-common";
import {Err, Ok} from "ts-results";
import {INTERNAL_ERROR} from "../NitroRepoAPI";

export async function installRequest(
  name: string,
  username: string,
  password: string,
  password_two: string,
  email: string
) {
  let installRequest = {
    name: name,
    username: username,
    email: email,
    password: password,
    password_two: password_two,
  };
  return http.post("install", installRequest).then(
    (result) => {
      const resultData = result.data;
      let value = JSON.stringify(resultData);

      let response: BasicResponse<unknown> = JSON.parse(value);

      if (response.success) {
        return Ok(true);
      } else {
        return Ok(false);
      }
    },
    (err) => {
      if (err.response) {
        return Err(INTERNAL_ERROR);
      } else if (err.request) {
        return Err(INTERNAL_ERROR);
      } else {
        return Err(INTERNAL_ERROR);
      }
    }
  );
}
