<template>
  <div class="cargoRepository">
    <div id="pullFromRepo">
      <h2>Pull from Repository</h2>
      <CodeMenu
        :snippets="snippetsForPull"
        defaultTab="config" />
    </div>
  </div>
</template>
<script setup lang="ts">
import { type RepositoryWithStorageName } from "@/types/repository";
import { type PropType } from "vue";
import { createSnippetsForPulling } from "./CargoRepositoryHelpers";

import CodeMenu from "@/components/core/code/CodeMenu.vue";

const props = defineProps({
  repository: {
    type: Object as PropType<RepositoryWithStorageName>,
    required: true,
  },
});

const snippetsForPull = createSnippetsForPulling(props.repository);
</script>
