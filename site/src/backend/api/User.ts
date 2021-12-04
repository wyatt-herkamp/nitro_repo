import { BasicResponse, DEFAULT_USER_LIST, User, UserList } from "../Response";
import http from "@/http-common";
import { Err, Ok, Result } from "ts-results";
import { APIError, INTERNAL_ERROR, INVALID_LOGIN } from "../NitroRepoAPI";
export interface AuthToken {
  id: number;
  user: number;
  token: string;
  expiration: number;
  created: number;
}
export async function getUser(token: string) {
  //${API_URL}
  const value = await http.get("/api/me", {
    headers: {
      Authorization: "Bearer " + token,
    },
  });

  if (value.status != 200) {
    return null;
  }
  const data = value.data as BasicResponse<unknown>;
  if (data.success) {
    return data.data as User;
  }

  return null;
}
export async function getUsers(token: string) {
  //${API_URL}
  const value = await http.get("/api/admin/user/list", {
    headers: {
      Authorization: "Bearer " + token,
    },
  });

  if (value.status != 200) {
    return DEFAULT_USER_LIST;
  }
  const data = value.data as BasicResponse<unknown>;
  if (data.success) {
    return data.data as UserList;
  }

  return DEFAULT_USER_LIST;
}

export async function getUserByID(
  token: string,
  id: number
): Promise<User | undefined> {
  //${API_URL}
  const value = await http.get("/api/admin/user/get/" + id, {
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

export async function login(username: string, password: string) {
  let loginRequest = {
    username: username,
    password: password,
  };
  return await http
    .post("api/login", loginRequest)
    .then((result) => {
      const resultData = result.data;
      let value = JSON.stringify(resultData);

      let response: BasicResponse<unknown> = JSON.parse(value);

      if (response.success) {
        let loginRequest = response as BasicResponse<AuthToken>;
        return Ok(loginRequest.data)
      } else {
        return Err(INVALID_LOGIN);
      }
    }, (err) => {
      if (err.response) {
        if (err.response.status == 401) {
          return Err(INVALID_LOGIN);
        } else if (err.response.status != 200) {
          return Err(INTERNAL_ERROR);
        }
      } else if (err.request) {
        return Err(INTERNAL_ERROR);
      } else {
        return Err(INTERNAL_ERROR);
      }
    });

}