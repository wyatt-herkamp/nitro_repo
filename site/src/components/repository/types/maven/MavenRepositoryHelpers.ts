import type { CodeSnippet } from '@/components/code/code'
import { apiURL } from '@/config'
import { type RepositoryWithStorageName } from '@/types/repository'
export function createSnippetsForPulling(
  repository: RepositoryWithStorageName
): Array<CodeSnippet> {
  return [createMavenSnippet(repository), createGradleKotlinSnippet(repository)]
}

function createMavenSnippet(repository: RepositoryWithStorageName): CodeSnippet {
  const baseURL = apiURL
  return {
    name: 'Maven',
    language: 'xml',
    key: 'maven',
    code: `
<repositories>
    <repository>
        <id>${repository.name}</id>
        <url>${baseURL}repositories/${repository.storage_name}/${repository.name}</url>
    </repository>
</repositories>
    `
  }
}

function createGradleKotlinSnippet(repository: RepositoryWithStorageName): CodeSnippet {
  const baseURL = apiURL
  return {
    name: 'Gradle Kotlin DSL',
    key: 'gradle-kotlin',
    language: 'kotlin',
    code: `
repositories {
    maven {
        url = uri("${baseURL}repositories/${repository.storage_name}/${repository.name}")
    }
}
    `
  }
}
