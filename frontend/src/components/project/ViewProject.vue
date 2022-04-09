<template>
  <div v-if="project != undefined">
    <div :class="child ? 'my-3' : 'flex flex-wrap'">
      <MavenProjectInfo
        :class="child ? '' : 'm-3 flex flex-col'"
        v-if="project.repo_summary.repo_type == 'maven'"
        :project="project"
        :child="child"
      />
    </div>
    <div :class="child ? 'my-3' : 'flex flex-wrap'">
      <ProjectBadge
        :class="child ? '' : 'm-3 flex flex-col'"
        :project="project"
        :child="child"
      />
    </div>
  </div>
</template>
<style scoped></style>
<script lang="ts">
import { Project } from "nitro_repo-api-wrapper";
import MavenProjectInfo from "@/components/project/types/maven/MavenProjectInfo.vue";
import { defineComponent, ref } from "vue";
import { useMeta } from "vue-meta";
import { useRouter } from "vue-router";
import ProjectBadge from "./badge/ProjectBadge.vue";

export default defineComponent({
  components: { MavenProjectInfo, ProjectBadge },
  props: {
    project: {
      required: true,
      type: Object as () => Project,
    },
    child: {
      default: false,
      type: Boolean,
    },
  },
  setup(props) {
    console.log(props.project == undefined);
    const router = useRouter();

    const { meta } = useMeta({
      title: props.project.version.name,
    });

    return {
      router,
    };
  },
});
</script>
