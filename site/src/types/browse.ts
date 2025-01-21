export interface RawBrowseResponse {
  files: RawBrowseFile[];
  project_resolution?: ProjectResolution;
}
export interface ProjectResolution {
  project_id?: string;
  version_id?: number;
}

export interface RawFile {
  name: string;
  mime_type: string;
  file_size: number;
  modified: string;
  created: string;
}
export interface RawDirectory {
  name: string;
  number_of_files: number;
}

export type RawBrowseFile =
  | {
      type: "File";
      value: RawFile;
    }
  | {
      type: "Directory";
      value: RawDirectory;
    };
export function fixCurrentPath(path: string): string {
  let newPath = path;
  if (newPath.startsWith("/")) {
    newPath = newPath.substring(1);
  }
  if (newPath.endsWith("/")) {
    newPath = newPath.substring(0, newPath.length - 1);
  }
  return newPath;
}
