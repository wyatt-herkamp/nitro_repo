<template>
  <CodeMenu :codes="snippets">
    <template v-slot:header>
      <div class="flex flex-row flex-warp">
        <div class="flex-grow">
          <h1 class="text-left text-white mt-5 ml-5 font-bold">
            Project Badge
          </h1>
        </div>
        <div class="mr-5">
          <img
            class="object-none my-5"
            :src="
              url +
              '/badge/' +
              project.repo_summary.storage +
              '/' +
              project.repo_summary.name +
              '/' +
              projectPath +
              '/badge'
            "
          />
        </div>
      </div>
    </template>
  </CodeMenu>
</template>

<script lang="ts">
import { computed, defineComponent, ref } from "vue";
import { useRouter } from "vue-router";
import { Project, Repository } from "@nitro_repo/nitro_repo-api-wrapper";
import { apiURL } from "@/http-common";
import { createProjectSnippet } from "@/api/repository/BadgeGen";

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
    const projectPath = props.project.version.name
      .replace(":", "/")
      .replace(".", "/");
    const snippets = createProjectSnippet(
      props.project.repo_summary.storage,
      props.project.repo_summary.name,
      projectPath,
      props.project.version.name
    );
    let page = ref(snippets[0].name);
    return { url, page, snippets, projectPath };
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
  @apply rounded-b-md;
  @apply bg-slate-800;
  @apply h-1/3;
  min-height: 100px;
  @apply mr-1;
}

.repositoryBadges {
  @apply relative;
  @apply flex;
  @apply rounded-t-md;
  @apply flex-col;
  @apply bg-slate-800;
  @apply h-4/5;
  @apply w-full;
  @apply xl:w-1/2;
  @apply 2xl:w-1/3;
}
</style>
