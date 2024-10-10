<template>
  <div class="mavenRepository">
    <div id="pullFromRepo">
      <h2>Pull from Repository</h2>
      <CodeMenu
        :snippets="snippetsForPull"
        defaultTab="maven" />
    </div>
  </div>
</template>
<script setup lang="ts">
import { type RepositoryWithStorageName } from "@/types/repository";
import { type PropType } from "vue";
import { createSnippetsForPulling } from "./MavenRepositoryHelpers";

import CodeMenu from "@/components/core/code/CodeMenu.vue";

const props = defineProps({
  repository: {
    type: Object as PropType<RepositoryWithStorageName>,
    required: true,
  },
});

const snippetsForPull = createSnippetsForPulling(props.repository);
</script>
<style lang="scss"></style>
<style lang="scss" scoped></style>
