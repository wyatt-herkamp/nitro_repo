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
      snippet: `implementation("${groupID}:${artifactID}:${version}")`,
    },
    {
      name: "Gradle.kts",
      lang: "kotlin",
      snippet: `implementation("${groupID}:${artifactID}:${version}")`,
    },
  ];
}
