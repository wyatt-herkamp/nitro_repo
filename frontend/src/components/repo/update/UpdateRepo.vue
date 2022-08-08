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
export default defineComponent({
  components: {
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
        schema_name: Record<string, unknown>;
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
