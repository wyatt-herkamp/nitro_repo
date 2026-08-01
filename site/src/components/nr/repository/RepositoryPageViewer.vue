<template>
  <div
    v-if="page.page_type == PageType.Markdown && markdownSource"
    id="pageContent">
    <vue-markdown :source="markdownSource" />
  </div>
</template>
<script setup lang="ts">
import { computed, type PropType } from "vue";
import { PageType, type RepositoryPage, type RepositoryWithStorageName } from "@/types/repository";
import VueMarkdown from "vue-markdown-render";

const props = defineProps({
  repository: {
    type: Object as PropType<RepositoryWithStorageName>,
    required: true,
  },
  page: {
    type: Object as PropType<RepositoryPage>,
    required: true,
  },
});

// `RepositoryPage.content` is optional — a repository can have a page configured with nothing in
// it yet — but `vue-markdown` requires a definite string. Rendering an empty page body is not
// useful, so an absent content also drops the wrapper entirely via the `v-if` above.
const markdownSource = computed(() => props.page.content ?? "");
</script>
<style lang="scss">
#pageContent {
  margin: 1rem;
}
</style>
