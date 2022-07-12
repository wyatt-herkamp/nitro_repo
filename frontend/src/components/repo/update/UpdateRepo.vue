<template>
  <Tabs v-model="view">
    <Tab name="General"> General </Tab>
    <Tab name="Frontend"> Frontend </Tab>
    <Tab name="Security"> Security </Tab>
    <Tab name="DeploySettings"> Deploy Settings </Tab>
    <Tab name="artifact">
      <template v-slot:icon>
        <DynamicIcon :repositoryType="repositoryType" />
      </template>
      {{ repositoryType }}
    </Tab>
    <Tab>
      <router-link
        :to="{
          name: 'ViewRepository',
          storage: repository.storage,
          repo: repository.name,
        }"
      >
        Repository Page</router-link
      ></Tab
    >
  </Tabs>
  <GeneralRepo v-if="view === 'General'" :repository="repository" />
  <SecurityRepo v-show="view === 'Security'" :repository="repository" />
  <FrontendRepo v-show="view === 'Frontend'" :repository="repository" />
  <DeployRepo v-show="view === 'DeploySettings'" :repository="repository" />
  <ArtifactSettings v-show="view === 'artifact'" :repository="repository" />
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
import ArtifactSettings from "@/components/repo/update/ArtifactSettings.vue";
import DynamicIcon from "@/components/repo/DynamicIcon.vue";
export default defineComponent({
  components: {
    DynamicIcon,
    ArtifactSettings,
    GeneralRepo,
    FrontendRepo,
    DeployRepo,
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

    useMeta({
      title: props.repository.name + " - " + view.value,
    });

    return {
      repositoryType,
      router,
      view,
      url,
    };
  },
});
</script>
