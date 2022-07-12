<template>
  <Tabs v-model="view">
    <Tab name="General"> General </Tab>
    <Tab name="Frontend"> Frontend </Tab>
    <Tab name="Security"> Security </Tab>
    <Tab name="DeploySettings"> Deploy Settings </Tab>

    <li class="flex flex-row items-center mx-auto min-w-fit">
      <ApachemavenIcon
        style="fill: white"
        v-show="repositoryType === 'Maven'"
      />
      <NpmIcon style="fill: white" v-show="repositoryType === 'NPM'" />
      <span class="text-sm font-medium text-quaternary">
        {{ repositoryType }}</span
      >
    </li>
  </Tabs>
  <SecurityRepo v-show="view === 'Security'" :repository="repository" />
  <FrontendRepo v-show="view === 'Frontend'" :repository="repository" />
  <GeneralRepo v-show="view === 'General'" :repository="repository" />
  <DeployRepo v-show="view === 'DeploySettings'" :repository="repository" />
</template>
<script lang="ts">
import { Repository } from "@nitro_repo/nitro_repo-api-wrapper";
import { defineComponent, ref } from "vue";
import { useMeta } from "vue-meta";
import { useRouter } from "vue-router";
import GeneralRepo from "@/components/repo/update/GeneralRepo.vue";
import FrontendRepo from "@/components/repo/update/FrontendRepo.vue";
import DeployRepo from "@/components/repo/update/DeployRepo.vue";
import SecurityRepo from "@/components/repo/update/SecurityRepo.vue";
import { apiURL } from "@/http-common";
import { ApachemavenIcon, NpmIcon } from "vue3-simple-icons";
export default defineComponent({
  components: {
    GeneralRepo,
    FrontendRepo,
    DeployRepo,
    ApachemavenIcon,
    NpmIcon,
    SecurityRepo,
  },
  props: {
    repository: {
      type: Object as () => Repository,
      required: true,
    },
  },
  setup(props) {
    const repositoryType = Object.keys(props.repository.repo_type)[0];

    const url = apiURL;

    const router = useRouter();
    const view = ref("General");

    const exampleBadgeURL = ref("");

    useMeta({
      title: props.repository.name,
    });

    return {
      repositoryType,
      exampleBadgeURL,
      router,
      view,
      url,
    };
  },
});
</script>
