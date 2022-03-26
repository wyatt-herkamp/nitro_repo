
import {Err, Ok} from "ts-results";
import {apiClient, BasicResponse, INTERNAL_ERROR, INVALID_LOGIN, NOT_AUTHORIZED,} from "../NitroRepoAPI";
import {AuthToken, User} from "./userTypes";

export async function login(username: string, password: string) {
  let loginRequest = {
    username: username,
    password: password,
  };
    return apiClient.post("api/login", loginRequest).then(
    (result) => {
      const resultData = result.data;
      let value = JSON.stringify(resultData);

      let response: BasicResponse<unknown> = JSON.parse(value);

      if (response.success) {
        let loginRequest = response as BasicResponse<AuthToken>;
        return Ok(loginRequest.data);
      } else {
        return Err(INVALID_LOGIN);
      }
    },
    (err) => {
      if (err.response) {
        if (err.response.status == 401) {
          return Err(INVALID_LOGIN);
        } else if (err.response.status != 200) {
          return Err(INTERNAL_ERROR);
        }else{
            return Err(INTERNAL_ERROR);

        }
      } else if (err.request) {
        return Err(INTERNAL_ERROR);
      } else {
        return Err(INTERNAL_ERROR);
      }
    }
  );
}

export async function updateMyPassword(password: string, token: string) {
  return apiClient
    .post(
      "/api/me/user/password",
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

export async function getUser(token: string) {
  return apiClient
    .get("/api/me", {
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
