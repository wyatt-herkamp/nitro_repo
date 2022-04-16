
<template>
  <CodeMenu :codes="snippets">
    <template v-slot:header>
      <div class="flex">
        <h1 class="text-left text-white mt-5 ml-5 font-bold">
          Repository Details
        </h1>
      </div>
    </template>
  </CodeMenu>
</template>

<script lang="ts">
import { computed, defineComponent, ref } from "vue";
import { useRouter } from "vue-router";
import { Repository } from "@nitro_repo/nitro_repo-api-wrapper";
import { apiURL } from "@/http-common";
import { PublicRepositoryInfo } from "@nitro_repo/nitro_repo-api-wrapper";
import createRepositoryInfo from "@/api/maven/CodeGen";

export default defineComponent({
  components: {},
  props: {
    repository: {
      required: true,
      type: Object as () => Repository | PublicRepositoryInfo,
    },
  },
  setup(props) {
    const url = apiURL;
    const repoURL =
      url + "/" + props.repository.storage + "/" + props.repository.name;
    const snippets = createRepositoryInfo(repoURL, props.repository.name);
    let page = ref(snippets[0].name);
    return { url, page, snippets };
  },

  methods: {
    changeViewValue(value: string) {
      this.$emit("changeView", value);
    },
  },
});
</script>
