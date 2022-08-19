export interface Version {
  installed: string;
  cargo_version: string;
  git_branch: string;
  git_commit: string;
  mode: string;
  build_timestamp: Date;
  rust_version: string;
  features: string[];
}
