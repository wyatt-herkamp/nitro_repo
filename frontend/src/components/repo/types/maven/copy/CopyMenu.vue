<template>
    <div class="repositoryDetails" :class="child ? 'min-w-full' : 'w-full xl:w-1/2 2xl:w-1/3'">
      <div class="flex-row h-1/3">
        <h1 class="text-left text-white mt-5 ml-5 font-bold">
          Repository details
        </h1>
        <nav class="flex flex-wrap p-6 m-1">
          <div
            v-for="repo in snippets"
            :key="repo.name"
            :class="page == repo.name ? 'active item' : 'item'"
            @click="page = repo.name"
          >
            {{ repo.name }}
          </div>
        </nav>
      </div>
      <template v-for="entry in snippets" :key="entry.name">
        <div v-if="entry.name === page">
          <div class="codeCube">
            <CodeViewComp :snippetInfo="entry" />
          </div>
        </div>
      </template>
    </div>
</template>

<script lang="ts">
import { computed, defineComponent, ref } from "vue";
import { useRouter } from "vue-router";
import { Repository } from "nitro_repo-api-wrapper";
import { apiURL } from "@/http-common";
import { PublicRepositoryInfo } from "nitro_repo-api-wrapper";
import CodeViewComp from "@/components/CodeViewComp.vue";
import createRepositoryInfo from "@/api/maven/CodeGen";

export default defineComponent({
  components: { CodeViewComp },
  props: {
    child: {
      default: false,
      type: Boolean,
    },
    repository: {
      required: true,
      type: Object as () => Repository | PublicRepositoryInfo,
    },
  },
  setup(props) {
    const url = apiURL;
    const repoURL =
      url + "/" + props.repository.storage + "/" + props.repository.name;
    const snippets = createRepositoryInfo(repoURL, props.repository.name);
    let page = ref(snippets[0].name);
    console.log(props.child);
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
.card-editor{
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