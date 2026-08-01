import type { CodeSnippet } from "@/components/core/code/code";
import type { Project } from "@/types/project";
import { createRepositoryRoute, type RepositoryWithStorageName } from "@/types/repository";

// Every template used to open with a bare newline and close with the indentation of its closing
// backtick, so each snippet rendered with a blank first line and a trailing run of spaces — and one
// of them indented `implementation(...)` by six. They are trimmed at the source rather than in the
// card, so what gets copied to the clipboard is exactly what is shown. (#498)

export function createSnippetsForPulling(
  repository: RepositoryWithStorageName,
): Array<CodeSnippet> {
  return [
    createMavenSnippet(repository),
    createGradleKotlinSnippet(repository),
    createGradleGroovySnippet(repository),
  ];
}

export function createProjectSnippets(
  project: Project,
  version: string = "{VERSION}",
): Array<CodeSnippet> {
  return [
    createMavenProjectSnippet(project, version),
    createGradleKotlinProjectSnippet(project, version),
    createGradleGroovyProjectSnippet(project, version),
  ];
}

export function createMavenProjectSnippet(project: Project, version: string): CodeSnippet {
  return {
    name: "Maven",
    language: "xml",
    key: "maven",
    code: `<dependency>
    <groupId>${project.scope}</groupId>
    <artifactId>${project.name}</artifactId>
    <version>${version}</version>
</dependency>`,
  };
}

export function createMavenSnippet(repository: RepositoryWithStorageName): CodeSnippet {
  const url = createRepositoryRoute(repository);

  return {
    name: "Maven",
    language: "xml",
    key: "maven",
    code: `<repositories>
    <repository>
        <id>${repository.name}</id>
        <url>${url}</url>
    </repository>
</repositories>`,
  };
}

export function createGradleKotlinSnippet(repository: RepositoryWithStorageName): CodeSnippet {
  const url = createRepositoryRoute(repository);
  return {
    name: "Gradle (Kotlin)",
    key: "gradle-kotlin",
    language: "kotlin",
    code: `repositories {
    maven {
        url = uri("${url}")
    }
}`,
  };
}

export function createGradleGroovySnippet(repository: RepositoryWithStorageName): CodeSnippet {
  const url = createRepositoryRoute(repository);
  return {
    name: "Gradle (Groovy)",
    key: "gradle-groovy",
    language: "groovy",
    code: `repositories {
    maven {
        url "${url}"
    }
}`,
  };
}

export function createGradleKotlinProjectSnippet(project: Project, version: string): CodeSnippet {
  return {
    name: "Gradle (Kotlin)",
    key: "gradle-kotlin",
    language: "kotlin",
    code: `implementation("${project.project_key}:${version}")`,
  };
}

export function createGradleGroovyProjectSnippet(project: Project, version: string): CodeSnippet {
  return {
    name: "Gradle (Groovy)",
    key: "gradle-groovy",
    language: "groovy",
    code: `implementation '${project.project_key}:${version}'`,
  };
}
