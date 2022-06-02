export interface Pom {
  project: Project;
}
export const xmlOptions = {
  ignoreAttributes: false,
};
export interface Project {
  modelVersion: string;
  groupId: string;
  artifactId: string;
  version: string;
}
