import {apiClient, BasicResponse} from "./NitroRepoAPI";
import {Err, Ok} from "ts-results";
import {INTERNAL_ERROR, NOT_AUTHORIZED,} from "./NitroRepoAPI";

export interface SiteInfo {
    name: string;
    description: string;
}


export async function getSiteInfo() {
    return apiClient
        .get("/api/info/site")
        .then(
            (result) => {
                const resultData = result.data;
                let value = JSON.stringify(resultData);

                let response: BasicResponse<unknown> = JSON.parse(value);

                if (response.success) {
                    return Ok(response.data as SiteInfo);
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
