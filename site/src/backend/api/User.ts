import axios from "axios";
import { BasicResponse, User } from "../Response";
import http from "@/http-common"
export async function getUser(token: string) {
    //${API_URL}
    const value = await http.get( "/api/me",
        {
            headers: {
                Authorization: "Bearer " + token
            }
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