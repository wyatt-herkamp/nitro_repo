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
import { defineComponent, ref } from "vue";

import { makeURL } from "@/http-common";
import createRepositoryInfo from "@/api/maven/CodeGen";
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
    const snippets = createRepositoryInfo(
      makeURL(
        `/repositories/${props.repository.storage}/${props.repository.name}`
      ),
      props.repository.name
    );
    const page = ref(snippets[0].name);
    return { page, snippets };
  },

  methods: {
    changeViewValue(value: string) {
      this.$emit("changeView", value);
    },
  },
});
</script>
