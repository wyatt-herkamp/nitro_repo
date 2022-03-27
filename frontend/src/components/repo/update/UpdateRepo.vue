<template>
  <div
    v-if="repository != undefined"
    class="min-h-screen w-full flex flex-wrap lg:flex-nowrap"
  >
    <div class="flex flex-col w-full">
      <EditMenu :repository="repository" @changeView="view = $event" />
      <div class="flex flex-col float-right w-auto">
        <GeneralRepo :repository="repository" v-if="view == 'General'" />
        <FrontendRepo :repository="repository" v-if="view == 'Frontend'" />
        <SecurityRepo :repository="repository" v-if="view == 'Security'" />
        <DeployRepo :repository="repository" v-if="view == 'Deploy'" />
      </div>
    </div>
    <div class="float-right lg:w-1/4 bg-slate-800">
      <ViewRepo :child="true" :repositoryType="repository" />
    </div>
  </div>
</template>
<style scoped>
.repositoryDetails {
  @apply min-w-full;
}
.toggle-bg:after {
  content: "";
  @apply absolute top-0.5 left-0.5 bg-white border border-gray-300 rounded-full h-5 w-5 transition shadow-sm;
}

input:checked + .toggle-bg:after {
  transform: translateX(100%);
  @apply border-white;
}

input:checked + .toggle-bg {
  @apply bg-blue-600 border-blue-600;
}
</style>
<script lang="ts">

import { getRepoByNameAndStorage } from "nitro_repo-api-wrapper";
import { Repository } from "nitro_repo-api-wrapper";
import EditMenu from "@/components/repo/edit/EditMenu.vue";
import ViewRepo from "@/components/repo/ViewRepo.vue";
import { defineComponent, ref } from "vue";
import { useCookie } from "vue-cookie-next";
import { useMeta } from "vue-meta";
import { useRoute, useRouter } from "vue-router";
import GeneralRepo from "@/components/repo/update/GeneralRepo.vue";
import FrontendRepo from "@/components/repo/update/FrontendRepo.vue";
import DeployRepo from "@/components/repo/update/DeployRepo.vue";
import SecurityRepo from "@/components/repo/update/SecurityRepo.vue";

export default defineComponent({
  components: {
    ViewRepo,
    EditMenu,
    GeneralRepo,
    FrontendRepo,
    DeployRepo,
    SecurityRepo,
  },
  setup() {
    const router = useRouter();
    const route = useRoute();
    let view = ref("General");

    let repository = ref<Repository | undefined>(undefined);
    const cookie = useCookie();
    const isLoading = ref(false);
    const exampleBadgeURL = ref("");
    const storage = route.params.storage as string;
    const repo = route.params.repo as string;
    const { meta } = useMeta({
      title: "Nitro Repo",
    });

    const getRepo = async () => {
      try {
        const value = (await getRepoByNameAndStorage(
          cookie.getCookie("token"),storage,repo
        )) as Repository;
        console.log(value);
        repository.value = value;

        meta.title = value.name;
      } catch (e) {
        console.log(e);
      }
    };
    getRepo();

    return {
      exampleBadgeURL,
      repository,
      router,
      view,
    };
  },
  methods: {
    handleClick(tab: any, event: any) {
      if (this.repository == undefined) return;
      if (tab.paneName === "upload") {
        this.router.replace(
          "/upload/" + this.repository.storage + "/" + this.repository.name
        );
      }
    },
  },
});
</script>
