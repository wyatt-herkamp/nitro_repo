<template>
  <NitroModal v-model="showLogin">
    <template v-slot:header>Login</template>
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

import LoginComp from "../user/LoginComp.vue";
import NitroModal from "../common/model/NitroModal.vue";

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
