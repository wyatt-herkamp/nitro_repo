<template>
  <CodeMenu  :codes="snippets">
    <template v-slot:header>
      <div class="grid grid-cols-2">
        <div>
          <h1 class="text-left text-white mt-5 ml-5 font-bold">
            Repository Badge
          </h1>
        </div>
        <div>
          <img
            class="object-none my-5"
            :src="
              url +
              '/badge/' +
              repository.storage +
              '/' +
              repository.name +
              '/nitro_repo_info/badge'
            "
          />
        </div>
      </div>
    </template>
  </CodeMenu>
</template>

<script lang="ts">
import { computed, defineComponent, ref } from "vue";
import { Repository } from "nitro_repo-api-wrapper";
import { apiURL } from "@/http-common";
import { PublicRepositoryInfo } from "nitro_repo-api-wrapper";
import { createBadgeSnippets } from "@/api/repository/BadgeGen";
import CodeMenu from "@/components/common/code/CodeMenu.vue";

export default defineComponent({
  components: { CodeMenu },
  props: {
    child: {
      default: false,
      type: Boolean,
    },
    repository: {
      required: true,
      type: Object as () => Repository | PublicRepositoryInfo,
    },
  },
  setup(props) {
    const url = apiURL;
    const snippets = createBadgeSnippets(
      props.repository.storage,
      props.repository.name
    );
    return { url, snippets };
  },
});
</script>
