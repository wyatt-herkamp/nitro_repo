import type { CodeSnippet } from "@/components/core/code/code";
import type { Project } from "@/types/project";
import { createRepositoryRoute, type RepositoryWithStorageName } from "@/types/repository";

/**
 * npm addresses a registry by URL, and a scoped package can be pointed at one independently of the
 * default. Both forms are offered because which one is right depends on whether the packages in
 * this registry share a scope.
 */
export function createSnippetsForPulling(
  repository: RepositoryWithStorageName,
): Array<CodeSnippet> {
  const url = createRepositoryRoute(repository);
  return [
    {
      name: ".npmrc",
      key: "npmrc",
      language: "ini",
      code: `registry=${url}/`,
    },
    {
      name: "Scoped .npmrc",
      key: "npmrc-scoped",
      language: "ini",
      code: `@your-scope:registry=${url}/`,
    },
    {
      name: "Login",
      key: "login",
      language: "bash",
      // The trailing slash matters: npm resolves package paths relative to the registry URL, and
      // without it the last path segment is replaced rather than appended.
      code: `npm login --registry=${url}/`,
    },
  ];
}

export function createProjectSnippets(
  project: Project,
  version: string = "latest",
): Array<CodeSnippet> {
  return [
    {
      name: "npm",
      key: "npm",
      language: "bash",
      code: `npm install ${project.project_key}@${version}`,
    },
    {
      name: "yarn",
      key: "yarn",
      language: "bash",
      code: `yarn add ${project.project_key}@${version}`,
    },
    {
      name: "pnpm",
      key: "pnpm",
      language: "bash",
      code: `pnpm add ${project.project_key}@${version}`,
    },
    {
      name: "package.json",
      key: "package-json",
      language: "json",
      code: `"dependencies": {
  "${project.project_key}": "^${version}"
}`,
    },
  ];
}
