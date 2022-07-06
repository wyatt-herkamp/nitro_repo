<template>
  <NitroModal v-model="showLogin">
    <template v-slot:header
      ><div class="py-2">
        <h1 class="text-quaternary text-3xl py-2">Welcome</h1>
        <h6 class="text-sm text-quaternary/50">
          Login to your Nitro Repo Account
        </h6>
      </div></template
    >
    <template v-slot:content><LoginComp /> </template>
  </NitroModal>
</template>
<style scoped>
:global(.loginButton) {
  @apply bg-slate-900;
}
</style>
<script lang="ts">
import { defineComponent, ref, watch } from "vue";

import LoginComp from "@/components//user/LoginComp.vue";
import NitroModal from "@/components/common/model/NitroModal.vue";

export default defineComponent({
  props: {
    modelValue: Boolean,
  },
  setup(props, { emit }) {
    const showLogin = ref(props.modelValue);
    watch(
      () => props.modelValue,
      (val) => {
        showLogin.value = val;
        emit("update:modelValue", val);
      }
    );
    watch(showLogin, (val) => {
      emit("update:modelValue", val);
    });
    return { showLogin };
  },
  components: { LoginComp, NitroModal },
});
</script>
