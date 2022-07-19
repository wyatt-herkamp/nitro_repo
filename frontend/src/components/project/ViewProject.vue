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
      <div class="grid grid-row-2 gap-4">
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
    const project = ref<Project | undefined>(undefined);
    await httpCommon.apiClient
      .get(
        `api/projects/${props.storage}/${props.repositoryName}/${props.projectName}`
      )
      .then((response) => {
        if (response.status == 200) {
          project.value = response.data;
        } else {
          //TODO handle 404
          console.error("Error fetching repository ");
        }
      });
    return {
      project,
    };
  },
});
</script>
