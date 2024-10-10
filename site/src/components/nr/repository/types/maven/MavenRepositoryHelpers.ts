import type { CodeSnippet } from "@/components/core/code/code";
import { apiURL } from "@/config";
import type { Project } from "@/types/project";
import { createRepositoryRoute, type RepositoryWithStorageName } from "@/types/repository";
export function createSnippetsForPulling(
  repository: RepositoryWithStorageName,
): Array<CodeSnippet> {
  return [createMavenSnippet(repository), createGradleKotlinSnippet(repository)];
}
export function createProjectSnippets(
  project: Project,
  version: string = "{VERSION}",
): Array<CodeSnippet> {
  return [
    createMavenProjectSnippet(project, version),
    createGradleKotlinProjectSnippet(project, version),
  ];
}
export function createMavenProjectSnippet(project: Project, version: string): CodeSnippet {
  return {
    name: "Maven",
    language: "xml",
    key: "maven",
    code: `
<dependency>
    <groupId>${project.scope}</groupId>
    <artifactId>${project.name}</artifactId>
    <version>${version}</version>
</dependency>
    `,
  };
}
export function createBlankLines(length: number): CodeSnippet {
  let code = "";
  for (let i = 0; i < length; i++) {
    code += "\n";
  }
  return {
    name: "Blank Lines",
    language: "xml",
    key: "blank-lines",
    code: code,
  };
}

export function createMavenSnippet(repository: RepositoryWithStorageName): CodeSnippet {
  const url = createRepositoryRoute(repository);

  return {
    name: "Maven",
    language: "xml",
    key: "maven",
    code: `
<repositories>
    <repository>
        <id>${repository.name}</id>
        <url>${url}</url>
    </repository>
</repositories>
    `,
  };
}

export function createGradleKotlinSnippet(repository: RepositoryWithStorageName): CodeSnippet {
  const url = createRepositoryRoute(repository);
  return {
    name: "Gradle Kotlin DSL",
    key: "gradle-kotlin",
    language: "kotlin",
    code: `
repositories {
    maven {
        url = uri("${url}")
    }
}`,
  };
}

export function createGradleKotlinProjectSnippet(project: Project, version: string): CodeSnippet {
  return {
    name: "Gradle Kotlin DSL",
    key: "gradle-kotlin",
    language: "kotlin",
    code: `
      implementation("${project.project_key}:${version}")
    `,
  };
}
