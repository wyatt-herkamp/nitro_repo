<template>
  <div class="settingContent">
    <h2 class="settingHeader">Repository Rules</h2>
  </div>
</template>
<script lang="ts">
import { defineComponent } from "vue";
import {Repository} from "@/types/repositoryTypes";

export default defineComponent({
  props: {
    repository: {
      required: true,
      type: Object as () => Repository,
    },
  },
  name: "NPMSettings",
});
</script>
