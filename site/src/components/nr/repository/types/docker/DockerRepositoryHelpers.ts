import type { CodeSnippet } from "@/components/core/code/code";
import type { Project } from "@/types/project";
import type { RepositoryWithStorageName } from "@/types/repository";

/**
 * The registry host and the image-name prefix a client must use.
 *
 * A Docker client cannot be given a URL path — `docker pull host/x/y` always requests
 * `https://host/v2/x/y/...`. So a repository is addressed one of two ways, and which one applies
 * depends on whether it has a hostname of its own:
 *
 *  - **hostname**: `docker pull docker.example.com/myimage` — the host identifies the repository,
 *    so image names carry no prefix;
 *  - **prefix**: `docker pull nitro.example.com/local/docker/myimage` — the first two segments of
 *    the image name are the storage and the repository.
 *
 * `hostname` is only reported when one is attached; otherwise the prefix form is the only one that
 * works, and showing a bare name would give the user a command that 404s.
 */
export interface DockerAddress {
  /** The registry host, without a scheme. */
  host: string;
  /** `{storage}/{repository}/`, or empty when the repository has its own hostname. */
  prefix: string;
}

export function dockerAddress(
  repository: RepositoryWithStorageName,
  hostname?: string,
): DockerAddress {
  if (hostname) {
    return { host: hostname, prefix: "" };
  }
  // The instance's own host, as the browser reached it — which is also the host a client on this
  // network can reach. Includes the port, which a registry URL needs.
  return {
    host: window.location.host,
    prefix: `${repository.storage_name}/${repository.name}/`,
  };
}

export function createSnippetsForPulling(
  repository: RepositoryWithStorageName,
  hostname?: string,
): Array<CodeSnippet> {
  const { host, prefix } = dockerAddress(repository, hostname);
  return [
    {
      name: "Login",
      key: "login",
      language: "bash",
      code: `docker login ${host}
# Username, and an auth token from Profile → Tokens as the password.`,
    },
    {
      name: "Pull",
      key: "pull",
      language: "bash",
      code: `docker pull ${host}/${prefix}IMAGE:TAG`,
    },
    {
      name: "Push",
      key: "push",
      language: "bash",
      code: `docker tag IMAGE:TAG ${host}/${prefix}IMAGE:TAG
docker push ${host}/${prefix}IMAGE:TAG`,
    },
  ];
}

export function createProjectSnippets(
  repository: RepositoryWithStorageName,
  project: Project,
  tag: string = "latest",
  hostname?: string,
): Array<CodeSnippet> {
  const { host, prefix } = dockerAddress(repository, hostname);
  const reference = `${host}/${prefix}${project.project_key}:${tag}`;
  return [
    {
      name: "Pull",
      key: "pull",
      language: "bash",
      code: `docker pull ${reference}`,
    },
    {
      name: "Run",
      key: "run",
      language: "bash",
      code: `docker run --rm ${reference}`,
    },
    {
      name: "Dockerfile",
      key: "dockerfile",
      language: "dockerfile",
      code: `FROM ${reference}`,
    },
  ];
}
