export interface SnippetInfo {
  name: string;
  grammar: string;
  lang: string;
  snippet: string;
}

export function escape(htmlStr: string): string {
  return htmlStr
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}
