<template>
  <div class="w-1/2 mx-auto">
    <h1 class="text-quaternary text-center text-3xl">Nitro Repository</h1>
    <FacebookIcon size="1.5x" />
    <div>
      <ArtifactSearch />
    </div>
  </div>
</template>

<script lang="js">
import { defineComponent } from "vue";
import ArtifactSearch from "@/components/ArtifactSearch.vue";
import { FacebookIcon } from 'vue-simple-icons'

export default defineComponent({
  components: {
    ArtifactSearch,FacebookIcon
  },
});
</script>
