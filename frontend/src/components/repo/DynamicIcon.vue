<template>
  <ApachemavenIcon style="fill: white" v-if="repositoryType === 'Maven'" />
  <NpmIcon style="fill: white" v-else-if="repositoryType === 'NPM'" />
</template>
<script lang="ts">
import { defineComponent } from "vue";
import { ApachemavenIcon, NpmIcon } from "vue3-simple-icons";

export default defineComponent({
  name: "DynamicIcon",
  components: { ApachemavenIcon, NpmIcon },
  props: {
    repositoryType: {
      required: true,
      type: String,
    },
  },
});
</script>
