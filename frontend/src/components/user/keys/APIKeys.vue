<template>
  <div class="md:w-1/2 mx-auto">
    <div class="flex flex-row my-2 justify-between">
      <h1 class="text-quaternary text-2xl ml-2">API Tokens</h1>
      <button
        class="bg-green-500 hover:bg-green-700 text-white font-bold py-2 px-4 h-12 rounded text-xs md:text-sm xxl:text-base"
        @click="$emit('newToken')"
      >
        Create new Token
      </button>
    </div>
    <ul class="w-full">
      <Token
        v-for="token in modelValue"
        v-bind:key="token.id"
        :myToken="token"
        @deleteToken="$emit('refresh')"
      />
    </ul>
  </div>
</template>
<script lang="ts">
import { defineComponent } from "vue";
import { AuthToken } from "@/types/userTypes";
import Token from "@/components/user/keys/Token.vue";

export default defineComponent({
  components: { Token },
  props: {
    modelValue: Object as () => Array<AuthToken>,
  },
  name: "APIKeys",
  emits: ["newToken", "refresh"],
  setup() {
    return {};
  },
});
</script>
