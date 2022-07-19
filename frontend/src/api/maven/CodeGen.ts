import { SnippetInfo } from "../CodeGenGeneral";

export default function createRepositoryInfo(
  url: string,
  name: string
): SnippetInfo[] {
  return [
    {
      name: "Maven",
      lang: "xml",
      grammar: "xml",
      snippet: `<repository>
    <id>${name}</id>
    <url>${url}</url>
</repository>`,
    },
    {
      name: "Gradle",
      lang: "groovy",
      grammar: "groovy",
      snippet: `maven {\n    url "${url}"\n}`.trim(),
    },
    {
      name: "Gradle.kts",
      lang: "kotlin",
      grammar: "kotlin",
      snippet: `maven("${url}")`,
    },
  ];
}
