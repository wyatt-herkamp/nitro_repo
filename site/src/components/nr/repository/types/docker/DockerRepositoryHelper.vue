<template>
  <div class="dockerRepository">
    <div id="pullFromRepo">
      <h2>Pull from Repository</h2>
      <CodeMenu
        :snippets="snippetsForPull"
        defaultTab="login" />
      <p
        v-if="!hostname"
        class="note">
        This registry has no hostname of its own, so image names must begin with
        <code>{{ repository.storage_name }}/{{ repository.name }}</code
        >. Attach a hostname under <strong>Hostnames</strong> to use bare image names.
      </p>
    </div>
  </div>
</template>
<script setup lang="ts">
import { type RepositoryWithStorageName, type RepositoryHostname } from "@/types/repository";
import { computed, ref, type PropType } from "vue";
import { createSnippetsForPulling } from "./DockerRepositoryHelpers";
import http from "@/http";

import CodeMenu from "@/components/core/code/CodeMenu.vue";

const props = defineProps({
  repository: {
    type: Object as PropType<RepositoryWithStorageName>,
    required: true,
  },
});

// Fetched rather than passed in: a Docker client's image names depend on whether the repository has
// a hostname, and the snippets are wrong — they 404 — if that is guessed rather than looked up.
const hostname = ref<string | undefined>(undefined);

async function loadHostname() {
  await http
    .get<RepositoryHostname[]>(`/api/repository/${props.repository.id}/hostnames`)
    .then((response) => {
      hostname.value = response.data[0]?.hostname;
    })
    .catch(() => {
      // A reader without permission to list hostnames still gets working snippets, in the
      // path-prefixed form that always works.
      hostname.value = undefined;
    });
}
loadHostname();

const snippetsForPull = computed(() => createSnippetsForPulling(props.repository, hostname.value));
</script>
<style scoped lang="scss">
.note {
  opacity: 0.8;
  font-size: 0.9rem;
}
</style>
