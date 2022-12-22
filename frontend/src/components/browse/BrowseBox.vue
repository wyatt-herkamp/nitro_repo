<template>
  <li class="link-box">
    <router-link
      class="link"
      :to="'/browse/' + file.full_path"
      v-if="file.directory"
    >
      <div>
        <font-awesome-icon icon="fa-folder" />
        {{ file.name }}
      </div>
    </router-link>
    <a class="link" :href="url + '/repositories/' + file.full_path" v-else>
      <div>
        <font-awesome-icon icon="fa-file-arrow-down" />
        {{ file.name }}
      </div>
    </a>
  </li>
</template>
<script lang="ts">
import { apiURL } from "@/http-common";
import { defineComponent } from "vue";
import { FileResponse } from "@/types/repositoryTypes";
export default defineComponent({
  props: {
    file: {
      type: Object as () => FileResponse,
      required: true,
    },
  },
  setup() {
    const url = apiURL;
    return { url };
  },
});
</script>
<style lang="scss" scoped>
.link-box {
  background: hsla(var(--color-secondary), 0.75);
  width: 100%;
  margin: 0.5rem;
  padding: 0.5rem;
  border-radius: 1rem;
  color: white;
  &:hover {
    background: hsla(var(--color-secondary), 1);
  }
  transition: background-color 0.2s ease-in-out;
}
</style>
