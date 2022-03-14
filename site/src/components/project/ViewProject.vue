<template>
  <div v-if="project != undefined">
    <div :class="child ? 'my-3' : 'flex flex-wrap'">
      <MavenProjectInfo1
        :class="child ? '' : 'm-3 flex flex-col'"
        v-if="project.repo_summary.repo_type == 'maven'"
        :project="project"
        :child="child"
      />
    </div>
  </div>
</template>
<style scoped></style>
<script lang="ts">
import { Project } from "@/backend/Response";
import MavenProjectInfo from "@/components/project/types/maven/MavenProjectInfo.vue";
import { defineComponent, ref } from "vue";
import { useMeta } from "vue-meta";
import { useRouter } from "vue-router";
import MavenProjectInfo1 from "@/components/project/types/maven/MavenProjectInfo.vue";

export default defineComponent({
  components: { MavenProjectInfo, MavenProjectInfo1 },
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
    const router = useRouter();

    const { meta } = useMeta({
      title: props.project.project.name,
    });

    return {
      router,
    };
  },
});
</script>
