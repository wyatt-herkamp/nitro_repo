<template>
  <Tabs v-model="view">
    <Tab name="General"> General </Tab>
    <Tab
      v-for="layout in repositoryLayout"
      v-bind:key="layout.config_name"
      :name="layout.config_name"
      >{{ layout.config_proper_name }}</Tab
    >
    <Tab name="">
      <router-link
        :to="{
          name: 'ViewRepository',
          params: {
            storage: repository.storage,
            repository: repository.name,
          },
        }"
      >
        Repository Page</router-link
      ></Tab
    >
  </Tabs>
  <GeneralRepo v-if="view === 'General'" :repository="repository" />
  <BadgeSettings v-else-if="view === 'badge'" :repository="repository" />
  <RepositoryPage
    v-else-if="view === 'repository_page'"
    :repository="repository"
  />
  <UndefinedSettingConfig
    v-bind:key="view"
    v-else
    :repository="repository"
    :settingName="view"
    :schema="getSchema()"
  />
</template>
<script lang="ts">
import { defineComponent, ref } from "vue";
import { useMeta } from "vue-meta";
import { useRouter } from "vue-router";
import GeneralRepo from "@/components/repo/update/GeneralRepo.vue";
import httpCommon, { apiURL } from "@/http-common";
import DynamicIcon from "@/components/repo/DynamicIcon.vue";
import { Repository } from "@/types/repositoryTypes";
import Tabs from "@/components/common/tabs/Tabs.vue";
import Tab from "@/components/common/tabs/Tab.vue";
import BadgeSettings from "@/components/repo/update/BadgeSettings.vue";
import RepositoryPage from "@/components/repo/update/RepositoryPage.vue";
import FrontendSettings from "@/components/repo/update/FrontendSettings.vue";
import UndefinedSettingConfig from "@/components/repo/update/UndefinedSettingConfig.vue";
import { JSONData } from "vanilla-jsoneditor";
export default defineComponent({
  components: {
    UndefinedSettingConfig,
    RepositoryPage,
    BadgeSettings,
    Tabs,
    Tab,
    DynamicIcon,
    GeneralRepo,
  },
  props: {
    repository: {
      type: Object as () => Repository,
      required: true,
    },
  },
  methods: {
    getSchema(): JSONData {
      return this.repositoryLayout.filter((layout) => {
        return layout.config_name === this.view;
      })[0].schema;
    },
  },
  setup(props) {
    const url = apiURL;

    const router = useRouter();
    const view = ref("General");
    useMeta({
      title: props.repository.name + " - " + view.value,
    });
    const repositoryLayout = ref<
      Array<{
        config_name: string;
        config_proper_name: string;
        schema: JSONData;
      }>
    >();
    httpCommon.apiClient
      .get(
        `api/admin/repositories/${props.repository.storage}/${props.repository.name}/layout`
      )
      .then((response) => {
        repositoryLayout.value = response.data;
      });

    return {
      router,
      view,
      url,
      repositoryLayout,
    };
  },
});
</script>
