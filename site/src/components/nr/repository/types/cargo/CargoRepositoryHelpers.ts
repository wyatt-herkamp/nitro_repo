import type { CodeSnippet } from "@/components/core/code/code";
import type { Project } from "@/types/project";
import { createRepositoryRoute, type RepositoryWithStorageName } from "@/types/repository";

/**
 * Cargo needs three separate things configured, and they go in three different files, so each is
 * offered on its own rather than as one block to be split up by hand:
 *
 *  - the registry itself, in `.cargo/config.toml`;
 *  - a token, which `cargo login` writes to `credentials.toml`;
 *  - `publish = ["nitro"]` in the crate's own manifest, without which `cargo publish` defaults to
 *    crates.io and refuses.
 */
export function createSnippetsForPulling(
  repository: RepositoryWithStorageName,
): Array<CodeSnippet> {
  const url = createRepositoryRoute(repository);
  return [
    {
      name: ".cargo/config.toml",
      key: "config",
      language: "toml",
      // The trailing slash matters: cargo appends the index path to this URL, and without it the
      // last segment is replaced rather than appended.
      code: `[registries.nitro]
index = "sparse+${url}/index/"`,
    },
    {
      name: "Login",
      key: "login",
      language: "bash",
      code: `cargo login --registry nitro
# Paste an auth token from Profile → Tokens.`,
    },
    {
      name: "Cargo.toml",
      key: "manifest",
      language: "toml",
      code: `[package]
# Without this, \`cargo publish\` targets crates.io.
publish = ["nitro"]`,
    },
  ];
}

export function createProjectSnippets(project: Project, version: string = "*"): Array<CodeSnippet> {
  return [
    {
      name: "cargo add",
      key: "cargo-add",
      language: "bash",
      code: `cargo add ${project.project_key}@${version} --registry nitro`,
    },
    {
      name: "Cargo.toml",
      key: "manifest",
      language: "toml",
      code: `[dependencies]
${project.project_key} = { version = "${version}", registry = "nitro" }`,
    },
  ];
}
