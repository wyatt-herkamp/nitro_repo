<template>
  <nav class="flex flex-wrap bg-slate-800 p-6 m-1 roun">
    <a @click="changeViewValue('General')">
      <EditItem> General </EditItem>
    </a>

    <a @click="changeViewValue('Frontend')">
      <EditItem> Frontend </EditItem>
    </a>
    <a @click="changeViewValue('Security')">
      <EditItem> Security </EditItem>
    </a>
    <a @click="changeViewValue('Deploy')">
      <EditItem> Deploy Settings </EditItem>
    </a>
    <img
      class="mx-2 py-1.5 rounded-lg font-bold px-6 m-1"
      :src="
        url +
        '/badge/' +
        repository.storage +
        '/' +
        repository.name +
        '/nitro_repo_status/badge.svg'
      "
    />
  </nav>
</template>

<script lang="ts">
import { defineComponent, ref } from "vue";
import { useRouter } from "vue-router";
import EditItem from "@/components/repo/edit/EditItem.vue";
import { Repository } from "@/backend/Response";
import { apiURL } from "@/http-common";

export default defineComponent({
  components: { EditItem },
  props: {
    repository: {
      required: true,
      type: Object as () => Repository,
    },
  },
  setup() {
    const url = apiURL;
    return { url };
  },
  methods: {
    changeViewValue(value: string) {
      console.log(value);
      this.$emit("changeView", value);
    },
  },
});
</script>
<style scoped>
.end {
  margin-left: auto;
}
</style>
