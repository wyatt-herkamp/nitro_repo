import { defineStore } from "pinia";
import { type Ref, ref } from "vue";
import http from "@/http";
import { Project, type RawProject } from "@/types/project";

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
    return {
      projects,
      getProjectById,
    };
  },
  {
    persist: false,
  },
);
