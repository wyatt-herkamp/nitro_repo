import type { CodeSnippet } from '@/components/code/code'
import { apiURL } from '@/config'
import type { Project } from '@/types/project'
import { createRepositoryRoute, type RepositoryWithStorageName } from '@/types/repository'
export function createSnippetsForPulling(
  repository: RepositoryWithStorageName
): Array<CodeSnippet> {
  return [createMavenSnippet(repository), createGradleKotlinSnippet(repository)]
}
export function createProjectSnippets(project: Project): Array<CodeSnippet> {
  // TODO: Versioning
  return [
    createMavenProjectSnippet(project, '1.0.0'),
    createGradleKotlinProjectSnippet(project, '1.0.0')
  ]
}
function createMavenProjectSnippet(project: Project, version: string): CodeSnippet {
  return {
    name: 'Maven',
    language: 'xml',
    key: 'maven',
    code: `
<dependency>
    <groupId>${project.scope}</groupId>
    <artifactId>${project.name}</artifactId>
    <version>${version}</version>
</dependency>
    `
  }
}

function createMavenSnippet(repository: RepositoryWithStorageName): CodeSnippet {
  const url = createRepositoryRoute(repository)

  return {
    name: 'Maven',
    language: 'xml',
    key: 'maven',
    code: `
<repositories>
    <repository>
        <id>${repository.name}</id>
        <url>${url}</url>
    </repository>
</repositories>
    `
  }
}

function createGradleKotlinSnippet(repository: RepositoryWithStorageName): CodeSnippet {
  const url = createRepositoryRoute(repository)
  return {
    name: 'Gradle Kotlin DSL',
    key: 'gradle-kotlin',
    language: 'kotlin',
    code: `
repositories {
    maven {
        url = uri("${url}")
    }
}
    `
  }
}

function createGradleKotlinProjectSnippet(project: Project, version: string): CodeSnippet {
  return {
    name: 'Gradle Kotlin DSL',
    key: 'gradle-kotlin',
    language: 'kotlin',
    code: `
      implementation("${project.project_key}:${version}")
    `
  }
}
