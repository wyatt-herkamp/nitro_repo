import { defineStore } from "pinia";
import { type Ref, ref } from "vue";
import http from "@/http";
import { Project, ProjectVersion, type RawProject, type RawProjectVersion } from "@/types/project";

export const useProjectStore = defineStore(
  "projects",
  () => {
    const projects: Ref<Record<string, Project>> = ref({});
    async function getProjectById(
      id: string,
      ignoreCache: boolean = true,
    ): Promise<Project | undefined> {
      if (ignoreCache || !projects.value[id]) {
        await http.get<RawProject>(`/api/project/${id}`).then((response) => {
          projects.value[id] = new Project(response.data);
        });
      }
      return projects.value[id];
    }
    async function getProjectByKey(
      repositoryId: string,
      projectKey: string,
    ): Promise<Project | undefined> {
      for (const project of Object.values(projects.value)) {
        if (project.repository_id === repositoryId && project.project_key === projectKey) {
          return project;
        }
      }
      return await http
        .get<RawProject>(`/api/project/by-key/${repositoryId}/${projectKey}`)
        .then((response) => {
          return new Project(response.data);
        })
        .catch(() => {
          return undefined;
        });
    }
    /**
     * Browsing resolves a `version_id` for a version directory but nothing else about it, so the
     * version has to be looked up before a snippet can name it.
     */
    async function getVersionById(versionId: string): Promise<ProjectVersion | undefined> {
      return await http
        .get<RawProjectVersion>(`/api/project/version/${versionId}`)
        .then((response) => {
          return new ProjectVersion(response.data);
        })
        .catch(() => {
          return undefined;
        });
    }
    return {
      projects,
      getProjectById,
      getProjectByKey,
      getVersionById,
    };
  },
  {
    persist: false,
  },
);
