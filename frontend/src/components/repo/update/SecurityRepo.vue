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
import { defineComponent, inject } from "vue";
import { Repository, setVisibility } from "@nitro_repo/nitro_repo-api-wrapper";

export default defineComponent({
  props: {
    repository: {
      required: true,
      type: Object as () => Repository,
    },
  },
  data() {
    const token = inject("token") as string;

    return { token };
  },
  methods: {
    async updateVisibility() {
      const response = await setVisibility(
        this.repository.storage,
        this.repository.name,
        this.repository.security.visibility,
        this.token
      );
      if (response.ok) {
        console.log(response.val.security.visibility);
        this.$notify({
          title: "Updated Repository",
          type: "info",
        });
      } else {
        this.$notify({
          title: "Unable Update Repository",
          text: JSON.stringify(response.val.user_friendly_message),
          type: "error",
        });
      }
    },
  },
});
</script>
