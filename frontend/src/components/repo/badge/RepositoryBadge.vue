<template>
  <CodeMenu :codes="snippets">
    <template v-slot:header>
      <div class="flex flex-row flex-warp">
        <div class="flex-grow">
          <h1 class="text-left text-white mt-5 ml-5 font-bold">
            Repository Badge
          </h1>
        </div>
        <div class="mr-5">
          <img
            class="object-none my-5"
            :src="
              makeURL(
                '/badge/repositories/' +
                  repository.storage +
                  '/' +
                  repository.name +
                  '/nitro_repo_badge'
              )
            "
            :alt="repository.storage + '/' + repository.name"
          />
        </div>
      </div>
    </template>
  </CodeMenu>
</template>

<script lang="ts">
import { defineComponent } from "vue";

import { makeURL } from "@/http-common";
import { createBadgeSnippets } from "@/api/repository/BadgeGen";
import CodeMenu from "@/components/common/code/CodeMenu.vue";

export default defineComponent({
  components: { CodeMenu },
  props: {
    repository: {
      required: true,
      type: Object as () => { name: string; storage: string },
    },
  },
  setup(props) {
    const snippets = createBadgeSnippets(
      props.repository.storage,
      props.repository.name
    );
    return { makeURL, snippets };
  },
});
</script>
