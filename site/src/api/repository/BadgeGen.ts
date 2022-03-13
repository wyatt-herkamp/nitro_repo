import { apiURL } from "@/http-common";
import { SnippetInfo } from "../CodeGenGeneral";

export default function createBadgeSnippets(storage: string, repository: string): SnippetInfo[] {
    const url = apiURL;
    const badgeURL=`${url}/badge/${storage}/${repository}/nitro_repo_info/badge.svg`;
    const appURL=`${url}/repositories/${storage}/${repository}/`;
    const text=`${repository} Repository`;
    return [
        {
            name: "Markdown",
            lang: "markdown",
            snippet: `[![${text}](${badgeURL})](${appURL})`,
        },
        {
            name: "html",
            lang: "html",
            snippet: `<a href=${appURL}>
            <img alt="${text}" src="${badgeURL}"/>
            </a>`,
        },
    ]
}