<template>
  <div class="settingContent">
    <h2 class="text-white m-3 text-left">Repository General Security</h2>

    <div class="flex flex-wrap mb-6 justify-center">
      <div class="settingBox">
        <label for="grid-policy">Page Provider</label>
        <select
          v-model="repository.security.visibility"
          @change="updateVisibility()"
          class="nitroTextInput"
        >
          <option value="Public">Public</option>
          <option value="Private">Private</option>
          <option value="Hidden">Hidden</option>
        </select>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent } from "vue";
import { Repository } from "@/types/repositoryTypes";

export default defineComponent({
  props: {
    repository: {
      required: true,
      type: Object as () => Repository,
    },
  },
  methods: {
    async updateVisibility() {
      // TODO update visibility
    },
  },
});
</script>
