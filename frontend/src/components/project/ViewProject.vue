<template>
  <div class="flex flex-row flex-wrap lg:flex-nowrap">
    <div class="lg:basis-3/4 m-4 text-quaternary">
      <div class="flex flex-row border-b-2">
        <h1 class="text-3xl">
          Project Description - {{ project.repo_summary.repository_type }}
        </h1>
        <DynamicIcon
          class="ml-5"
          :repositoryType="project.repo_summary.repository_type"
        />
      </div>
      <div
        class="mt-5 bg-secondary bg-slate-800 rounded-md p-2"
        v-show="project.frontend_response !== ''"
        v-html="project.frontend_response"
      ></div>
      <div
        class="mt-5 bg-secondary bg-slate-800 rounded-md p-2"
        v-show="project.frontend_response === ''"
      >
        <p class="text-center text-quaternary">No description available.</p>
      </div>
    </div>
    <div class="lg:basis-1/4">
      <div class="grid grid-rows-3 gap-4">
        <div class="m-2 bg-slate-800">
          <h1 class="text-white mt-5 ml-5 font-bold">Project Info</h1>
          <div class="text-white mt-5 ml-5">
            <span
              >Last Updated On {{ last_updated_date }} at
              {{ last_updated_time }}</span
            >
          </div>
        </div>

        <ProjectBadge
          class="m-2"
          :project="{
            storage: storage,
            repository: repositoryName,
            project: projectName,
            version: version,
          }"
        />

        <MavenProjectInfo
          class="m-2"
          v-if="project.repo_summary.repository_type === 'Maven'"
          :project="project"
        />
      </div>
    </div>
  </div>
</template>
<style scoped></style>
<script lang="ts">
import MavenProjectInfo from "@/components/project/types/maven/MavenProjectInfo.vue";
import { defineComponent, ref } from "vue";
import { useMeta } from "vue-meta";
import ProjectBadge from "./badge/ProjectBadge.vue";
import { Project } from "@/types/repositoryTypes";
import httpCommon from "@/http-common";
import DynamicIcon from "@/components/repo/DynamicIcon.vue";

export default defineComponent({
  components: { MavenProjectInfo, ProjectBadge, DynamicIcon },
  props: {
    storage: {
      type: String,
    },
    repositoryName: {
      type: String,
    },
    projectName: {
      type: String,
    },
    version: {
      type: String,
      default: "",
    },
  },
  async setup(props) {
    useMeta({
      title: props.version,
    });
    const project: Project = await httpCommon.apiClient
      .get(
        `api/projects/${props.storage}/${props.repositoryName}/${props.projectName}`
      )
      .then((response) => {
        return response.data;
      })
      .catch((error) => {
        console.error(error);
        return {
          repo_summary: {
            name: "",
            storage: "",
            page_provider: "",
            repo_type: "",
            visibility: "",
          },
          frontend_response: "",
          backend_response: "",
          version: "",
          last_updated: 0,
        };
      });
    const last_updated_date = new Date(
      project.version.created
    ).toLocaleDateString("en-US");
    const last_updated_time = new Date(
      project.version.created
    ).toLocaleTimeString("en-US");
    return {
      project,
      last_updated_date,
      last_updated_time,
    };
  },
});
</script>
