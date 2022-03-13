<template>
  <div v-if="repository != undefined">
    <ViewRepo :storage="storage" :repository="repository" />
  </div>
</template>

<script lang="ts">
import ViewRepo from "@/components/repo/ViewRepo.vue";
import { defineComponent } from "vue";
import { useRoute } from "vue-router";

export default defineComponent({
  components: { ViewRepo },
  setup() {
    const route = useRoute();

    const storage = route.params.storage as string;
    const repository = route.params.repo as string;

    return {
      storage,
      repository,
    };
  },
});
</script>