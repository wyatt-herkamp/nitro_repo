<template>
  <div class="w-1/2 mx-auto">
    <h1 class="text-quaternary text-center text-3xl">Nitro Repository</h1>

    <div>
      <ArtifactSearch />
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent } from "vue";
import ArtifactSearch from "@/components/ArtifactSearch.vue";
export default defineComponent({
  setup() {
    return {};
  },
  components: {
    ArtifactSearch,
  },
});
</script>
