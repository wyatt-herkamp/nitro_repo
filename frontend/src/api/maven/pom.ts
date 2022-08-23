export interface Pom {
  project: Project;
}

export interface Project {
  modelVersion: string;
  groupId: string;
  artifactId: string;
  version: string;
}
export const xmlOptions = {
  ignoreAttributes: false,
};
