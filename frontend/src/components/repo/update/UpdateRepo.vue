<template>
  <Tabs>
    <Tab name="General">
      <GeneralRepo :repository="repository" />
    </Tab>
    <Tab name="Frontend">
      <FrontendRepo :repository="repository" />
    </Tab>
    <Tab name="Security">
      <SecurityRepo :repository="repository" />
    </Tab>
    <Tab name="Deploy Settings">
      <DeployRepo :repository="repository" />
    </Tab>
  </Tabs>
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
import Tab from "@/components/common/tabs/Tab.vue";
import Tabs from "@/components/common/tabs/Tabs.vue";

export default defineComponent({
  components: {
    GeneralRepo,
    FrontendRepo,
    DeployRepo,
    Tabs,
    Tab,

    SecurityRepo,
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

    const exampleBadgeURL = ref("");

    useMeta({
      title: props.repository.name,
    });

    return {
      exampleBadgeURL,
      router,
      view,
      url,
    };
  },
});
</script>
