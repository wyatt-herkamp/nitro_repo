<template>
  <div
    v-if="repository != undefined"
    class="min-h-screen w-full flex flex-wrap lg:flex-nowrap"
  >
    <div class="flex flex-col float-right w-full lg:w-auto bg-slate-800">
      <ViewRepo :storage="storage" :repository="repository" />
    </div>
  </div>
</template>

<script lang="ts">
import { getRepoByID } from "@/backend/api/Repository";
import { Repository } from "@/backend/Response";
import EditMenu from "@/components/repo/edit/EditMenu.vue";
import ViewRepo from "@/components/repo/ViewRepo.vue";
import { defineComponent, ref } from "vue";
import { useCookie } from "vue-cookie-next";
import { useMeta } from "vue-meta";
import { useRoute, useRouter } from "vue-router";

export default defineComponent({
  components: { ViewRepo, EditMenu },
  setup() {
    const router = useRouter();
    const route = useRoute();

    const exampleBadgeURL = ref("");
    const storage = route.params.storage as string;
    const repository = route.params.repo as string;

    return {
      storage,
      repository,
      storage,
    };
  },
  methods: {},
});
</script>