<template>
  <li class="w-full bg-quaternary rounded-xl my-2 p-4 flex flex-col">
    <div class="flex flex-row justify-between">
      <div class="flex flex-col">
        <span
          v-if="myToken.properties.description"
          class="inline-flex items-center"
          >{{ myToken.properties.description }}</span
        >
        <span v-else class="basis-11/12 inline-flex items-center">
          No description available.
        </span>
      </div>
      <div>
        <button
          @click="deleteToken"
          class="bg-red-500 hover:bg-red-700 text-white font-bold py-2 px-4 h-12 rounded"
        >
          Delete
        </button>
      </div>
    </div>

    <span class="text-xs">ID: {{ myToken.id }} Created: {{ date }}</span>
  </li>
</template>
<script lang="ts">
import { defineComponent } from "vue";
import { AuthToken } from "@/types/userTypes";
import httpCommon from "@/http-common";

export default defineComponent({
  props: {
    myToken: {
      required: true,
      type: Object as () => AuthToken,
    },
  },
  emits: ["deleteToken"],
  name: "Token",
  setup(props) {
    const date = new Date(props.myToken.created).toLocaleDateString("en-US");

    return { date };
  },
  methods: {
    async deleteToken() {
      await httpCommon.apiClient
        .delete(`api/token/${this.myToken.id}`)
        .then((response) => {
          if (response.status === 204) {
            this.$emit("deleteToken");
          } else {
            console.error(response);
            this.$notify({
              type: "error",
              title: "Could not Delete Token",
            });
          }
        });
    },
  },
});
</script>
