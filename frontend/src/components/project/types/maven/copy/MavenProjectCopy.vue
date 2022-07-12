<template>
  <CodeMenu :codes="snippets">
    <template v-slot:header>
      <h1 class="text-left text-white mt-5 ml-5 font-bold">Project Details</h1>
    </template>
  </CodeMenu>
</template>

<script lang="ts">
import { defineComponent, ref } from "vue";
import { Project } from "@nitro_repo/nitro_repo-api-wrapper";
import { apiURL } from "@/http-common";
import createProjectGen from "@/api/maven/ProjectGen";

export default defineComponent({
  components: {},
  props: {
    project: {
      required: true,
      type: Object as () => Project,
    },
  },
  setup(props) {
    const url = apiURL;
    const value = props.project.version.name.split(":");
    const snippets = createProjectGen(
      value[0],
      value[1],
      props.project.project.versions.latest_release
    );
    const page = ref(snippets[0].name);
    return { url, page, snippets };
  },

  methods: {
    changeViewValue(value: string) {
      console.log(value);
      this.$emit("changeView", value);
    },
  },
});
</script>
<style scoped>
.active {
  @apply text-yellow-50 !important;
  @apply cursor-default !important;
  @apply border-slate-900 !important;
}

.item {
  @apply text-white;
  @apply py-4;
  @apply px-7;
  @apply flex-grow;
  @apply text-center;
  @apply border-b-2;
  @apply cursor-pointer;
  @apply border-transparent;
}
.codeCube {
  min-height: 100px;
  @apply m-0;
}
.card-editor {
  @apply w-full;
  @apply m-0;
}
.repositoryDetails {
  @apply flex;
  @apply rounded-t-md;
  @apply flex-col;
  @apply bg-slate-800;
  @apply h-4/5;
}
</style>
