import { apiURL, makeURL } from "@/http-common";
import { escapeHtml, SnippetInfo } from "../CodeGenGeneral";

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
      snippet: `<a href="${appURL}">
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
  const badgeURL = makeURL(
    `badge/repositories/${storage}/${repository}/${project}`
  );
  const appURL = makeURL(`project/${storage}/${repository}/${project}`);
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
      snippet: `<a href=${appURL}>
  <img alt="${text}" src="${badgeURL}"/>
</a>`,
    },
  ];
}
