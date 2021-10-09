import http from "@/http-common";
import { BasicResponse, StorageList, DEFAULT_STORAGE_LIST } from "../Response";
export async function getStorages(token: string) {
    const value = await http.get( "/api/storages/list",
        {
            headers: {
                Authorization: "Bearer " + token
            }
        });

    if (value.status != 200) {
        return DEFAULT_STORAGE_LIST;
    }
    const data = value.data as BasicResponse<unknown>;
    if (data.success) {
        return data.data as StorageList;
    }

    return DEFAULT_STORAGE_LIST;
}
