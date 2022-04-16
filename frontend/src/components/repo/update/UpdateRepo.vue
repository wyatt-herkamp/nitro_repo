<template>
  <div v-if="repository != undefined">
    <div class="mx-auto">
      <SubNavBar v-model="view">
        <SubNavItem index="General"> General </SubNavItem>
        <SubNavItem index="Frontend"> Frontend </SubNavItem>
        <SubNavItem index="Security"> Security </SubNavItem>
        <SubNavItem index="Deploy"> Deploy Settings </SubNavItem>
        <SubNavItem index="Badge" :disabled="true">
          <img
            class="mx-auto font-bold px-6 m-1"
            :src="
              url +
              '/badge/' +
              repository.storage +
              '/' +
              repository.name +
              '/nitro_repo_status/badge'
            "
          />
        </SubNavItem>
      </SubNavBar>
      <div class="w-auto m-auto">
        <GeneralRepo :repository="repository" v-if="view == 'General'" />
        <FrontendRepo :repository="repository" v-if="view == 'Frontend'" />
        <SecurityRepo :repository="repository" v-if="view == 'Security'" />
        <DeployRepo :repository="repository" v-if="view == 'Deploy'" />
      </div>
    </div>
  </div>
</template>
<script lang="ts">
import { getRepoByNameAndStorage } from "nitro_repo-api-wrapper";
import { Repository } from "nitro_repo-api-wrapper";
import ViewRepo from "@/components/repo/ViewRepo.vue";
import { defineComponent, inject, ref } from "vue";
import { useMeta } from "vue-meta";
import { useRoute, useRouter } from "vue-router";
import GeneralRepo from "@/components/repo/update/GeneralRepo.vue";
import FrontendRepo from "@/components/repo/update/FrontendRepo.vue";
import DeployRepo from "@/components/repo/update/DeployRepo.vue";
import SecurityRepo from "@/components/repo/update/SecurityRepo.vue";
import { apiURL } from "@/http-common";

export default defineComponent({
  components: {
    ViewRepo,
    GeneralRepo,
    FrontendRepo,
    DeployRepo,
    SecurityRepo,
  },
  setup() {
    const url = apiURL;

    const router = useRouter();
    let view = ref("General");

    let repository = ref<Repository | undefined>(undefined);
    const token: string | undefined = inject("token");
    if (token == undefined) {
      useRouter().push("login");
    }
    const exampleBadgeURL = ref("");
    const route = useRoute();

    const storage = route.params.storage as string;
    const repo = route.params.repo as string;
    const { meta } = useMeta({
      title: "Nitro Repo",
    });

    const getRepo = async () => {
      try {
        const value = (await getRepoByNameAndStorage(
          token as string,
          storage,
          repo
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
      url,
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
