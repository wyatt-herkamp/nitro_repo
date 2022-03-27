<template>
  <ViewProject v-if="project != undefined" :project="project" />
</template>

<script lang="ts">
import { getProject } from "nitro_repo-api-wrapper";
import { Project } from "nitro_repo-api-wrapper";
import { defineComponent, ref } from "vue";
import { useCookie } from "vue-cookie-next";
import { useRoute } from "vue-router";
import ViewProject from "@/components/project/ViewProject.vue";

export default defineComponent({
  components: { ViewProject },
  setup() {
    const route = useRoute();
    const storage = route.params.storage as string;
    const repository = route.params.repo as string;
    const id = route.params.id as string;
    let version = route.params.version as string;
    let cookie = useCookie();
    const project = ref<Project | undefined>(undefined);
    const getInfo = async () => {
      let value = await getProject(
        cookie.getCookie("token"),
        storage,
        repository,
        id,
        version
      );
      console.log(value);
      project.value = value;
    };
    getInfo();
    return { project };
  },
});
</script>
