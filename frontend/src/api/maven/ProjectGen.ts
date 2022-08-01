import { SnippetInfo } from "../CodeGenGeneral";

export default function createProjectGen(
  groupID: string,
  artifactID: string,
  version: string
): SnippetInfo[] {
  return [
    {
      name: "Maven",
      lang: "xml",
      grammar: "xml",
      snippet: `
<dependency>
    <groupId>${groupID}</groupId>
    <artifactId>${artifactID}</artifactId>
    <version>${version}</version>
</dependency>`.trim(),
    },
    {
      name: "Gradle",
      lang: "groovy",
      grammar: "groovy",
      snippet: `implementation("${groupID}:${artifactID}:${version}")`,
    },
    {
      name: "Gradle.kts",
      lang: "kotlin",
      grammar: "kotlin",
      snippet: `implementation("${groupID}:${artifactID}:${version}")`,
    },
  ];
}
