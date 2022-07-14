<template>
  <Tabs v-model="view">
    <Tab name="General"> General </Tab>
    <Tab name="Frontend"> Frontend </Tab>
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
  <FrontendRepo
    v-if="badgeSettings && frontendSettings"
    v-show="view === 'Frontend'"
    :frontendSettings="frontendSettings"
    :badgeSettings="badgeSettings"
    :repository="repository"
  />
  <ArtifactSettings v-show="view === 'artifact'" :repository="repository" />
</template>
<script lang="ts">
import { defineComponent, ref } from "vue";
import { useMeta } from "vue-meta";
import { useRouter } from "vue-router";
import GeneralRepo from "@/components/repo/update/GeneralRepo.vue";
import FrontendRepo from "@/components/repo/update/FrontendRepo.vue";
import { apiURL } from "@/http-common";
import ArtifactSettings from "@/components/repo/update/ArtifactSettings.vue";
import DynamicIcon from "@/components/repo/DynamicIcon.vue";
import { Frontend, BadgeSettings, Repository } from "@/types/repositoryTypes";
export default defineComponent({
  components: {
    DynamicIcon,
    ArtifactSettings,
    GeneralRepo,
    FrontendRepo,
  },
  props: {
    repository: {
      type: Object as () => Repository,
      required: true,
    },
  },
  setup(props) {
    const repositoryType = Object.keys(props.repository.repository_type)[0];
    const frontendSettings = ref<Frontend | undefined>(undefined);
    const badgeSettings = ref<BadgeSettings | undefined>(undefined);
    const url = apiURL;

    const router = useRouter();
    const view = ref("General");

    useMeta({
      title: props.repository.name + " - " + view.value,
    });

    return {
      repositoryType,
      frontendSettings,
      badgeSettings,
      router,
      view,
      url,
    };
  },
});
</script>
