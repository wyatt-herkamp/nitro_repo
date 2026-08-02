/**
 * File-type icons for the repository browser. (#497)
 *
 * The icon logic was previously, in full:
 *
 *     const fileIcon = computed(() => {
 *       // TODO: More icons
 *       return "fa-solid fa-file";
 *     });
 *
 * so every entry in every repository — jars, POMs, checksums, signatures — drew the same grey page.
 * In an artifact browser the *kind* of file is most of the information in a row: the difference
 * between `.jar`, `.pom` and `.jar.sha1` is what someone is scanning for.
 *
 * Colours follow the Atom Material palette the issue asks for, expressed as tokens so both themes
 * get a readable version.
 */

export interface FileIcon {
  icon: string;
  /** A `--file-*` custom property defined in `browse.scss`. */
  color: string;
  /** What the type is, for the row's tooltip. */
  label: string;
}

const DEFAULT_ICON: FileIcon = { icon: "file", color: "--file-default", label: "File" };

/**
 * Longest suffix wins, so `.jar.sha1` is a checksum rather than a jar. Ordering in this table does
 * not matter — the lookup sorts by length — but grouping does, for anyone adding to it.
 */
const BY_EXTENSION: Record<string, FileIcon> = {
  // Java / Maven
  jar: { icon: "box-archive", color: "--file-archive", label: "Java archive" },
  war: { icon: "box-archive", color: "--file-archive", label: "Web archive" },
  ear: { icon: "box-archive", color: "--file-archive", label: "Enterprise archive" },
  aar: { icon: "box-archive", color: "--file-archive", label: "Android archive" },
  klib: { icon: "box-archive", color: "--file-archive", label: "Kotlin library" },
  pom: { icon: "file-code", color: "--file-config", label: "Maven POM" },
  module: { icon: "file-code", color: "--file-config", label: "Gradle module metadata" },

  // Checksums and signatures. These are the noisiest rows in a Maven repository, so they get a
  // deliberately quiet colour.
  sha1: { icon: "fingerprint", color: "--file-checksum", label: "SHA-1 checksum" },
  sha256: { icon: "fingerprint", color: "--file-checksum", label: "SHA-256 checksum" },
  sha512: { icon: "fingerprint", color: "--file-checksum", label: "SHA-512 checksum" },
  md5: { icon: "fingerprint", color: "--file-checksum", label: "MD5 checksum" },
  asc: { icon: "file-shield", color: "--file-signature", label: "PGP signature" },
  sig: { icon: "file-shield", color: "--file-signature", label: "Signature" },

  // npm
  tgz: { icon: "box-archive", color: "--file-archive", label: "npm tarball" },
  // Cargo. A `.crate` is a gzipped tar, so it belongs with the archives rather than looking like
  // an unknown binary.
  crate: { icon: "box-archive", color: "--file-archive", label: "Rust crate" },
  tar: { icon: "box-archive", color: "--file-archive", label: "Tarball" },
  zip: { icon: "box-archive", color: "--file-archive", label: "Zip archive" },
  gz: { icon: "box-archive", color: "--file-archive", label: "Gzip archive" },

  // Source and docs
  java: { icon: "file-code", color: "--file-source", label: "Java source" },
  kt: { icon: "file-code", color: "--file-source", label: "Kotlin source" },
  scala: { icon: "file-code", color: "--file-source", label: "Scala source" },
  groovy: { icon: "file-code", color: "--file-source", label: "Groovy source" },
  js: { icon: "file-code", color: "--file-source", label: "JavaScript" },
  ts: { icon: "file-code", color: "--file-source", label: "TypeScript" },
  rs: { icon: "file-code", color: "--file-source", label: "Rust source" },

  json: { icon: "file-code", color: "--file-data", label: "JSON" },
  xml: { icon: "file-code", color: "--file-config", label: "XML" },
  yaml: { icon: "file-code", color: "--file-config", label: "YAML" },
  yml: { icon: "file-code", color: "--file-config", label: "YAML" },
  toml: { icon: "file-code", color: "--file-config", label: "TOML" },
  properties: { icon: "file-code", color: "--file-config", label: "Properties" },

  md: { icon: "file-lines", color: "--file-doc", label: "Markdown" },
  txt: { icon: "file-lines", color: "--file-doc", label: "Text" },
  html: { icon: "file-code", color: "--file-markup", label: "HTML" },
  pdf: { icon: "file-pdf", color: "--file-doc", label: "PDF" },

  png: { icon: "file-image", color: "--file-image", label: "Image" },
  jpg: { icon: "file-image", color: "--file-image", label: "Image" },
  jpeg: { icon: "file-image", color: "--file-image", label: "Image" },
  gif: { icon: "file-image", color: "--file-image", label: "Image" },
  svg: { icon: "file-image", color: "--file-image", label: "Image" },
};

/** Falls back on the mime type when the extension says nothing. */
const BY_MIME_PREFIX: Array<[string, FileIcon]> = [
  ["image/", { icon: "file-image", color: "--file-image", label: "Image" }],
  ["text/html", { icon: "file-code", color: "--file-markup", label: "HTML" }],
  ["text/", { icon: "file-lines", color: "--file-doc", label: "Text" }],
  ["application/json", { icon: "file-code", color: "--file-data", label: "JSON" }],
  ["application/xml", { icon: "file-code", color: "--file-config", label: "XML" }],
  ["application/zip", { icon: "box-archive", color: "--file-archive", label: "Archive" }],
  ["application/java", { icon: "box-archive", color: "--file-archive", label: "Java archive" }],
];

const SORTED_EXTENSIONS = Object.keys(BY_EXTENSION).sort((a, b) => b.length - a.length);

export function iconForFile(name: string, mimeType?: string): FileIcon {
  const lower = name.toLowerCase();

  // Suffix match rather than "everything after the last dot", so a compound extension like
  // `.jar.sha1` resolves to the checksum and `maven-metadata.xml.md5` does not read as XML.
  for (const extension of SORTED_EXTENSIONS) {
    if (lower.endsWith(`.${extension}`)) {
      return BY_EXTENSION[extension] ?? DEFAULT_ICON;
    }
  }

  if (mimeType) {
    const mime = mimeType.toLowerCase();
    for (const [prefix, icon] of BY_MIME_PREFIX) {
      if (mime.startsWith(prefix)) return icon;
    }
  }

  return DEFAULT_ICON;
}
