import { apiURL } from "@/http-common";
import { SnippetInfo } from "../CodeGenGeneral";

export function createBadgeSnippets(
  storage: string,
  repository: string
): SnippetInfo[] {
  const url = apiURL;
  const badgeURL = `${url}badge/${storage}/${repository}/nitro_repo_info/badge`;
  const appURL = `${url}repositories/${storage}/${repository}/`;
  const text = `${repository} Repository`;
  return [
    {
      name: "Markdown",
      grammar: "markdown",
      lang: "markdown",
      snippet: `[![${text}](${badgeURL})](${appURL})`,
    },
    {
      name: "html",
      grammar: "html",
      lang: "html",
      snippet: `<a href=${appURL}>
      <img alt="${text}" src="${badgeURL}"/>
</a>`,
    },
  ];
}

export function createProjectSnippet(
  storage: string,
  repository: string,
  project: string,
  projectName: string
): SnippetInfo[] {
  const url = apiURL;
  const badgeURL = `${url}badge/${storage}/${repository}/${project}/badge`;
  const appURL = `${url}project/${storage}/${repository}/${projectName}`;
  const text = `${repository} Repository`;
  return [
    {
      name: "Markdown",
      lang: "markdown",
      grammar: "markdown",
      snippet: `[![${text}](${badgeURL})](${appURL})`,
    },
    {
      name: "html",
      lang: "html",
      grammar: "html",
      snippet: `<a href=${appURL}><img alt="${text}" src="${badgeURL}"/></a>`,
    },
  ];
}
