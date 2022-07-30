<template>
  <div v-bind="$attrs" class="settingContent">
    <slot />
    <div class="settingBox">
      <label class="nitroLabel" for="grid-password"> Password </label>
      <input
        class="nitroTextInput"
        id="grid-password"
        type="password"
        v-model="password"
      />
      <label class="nitroLabel" for="grid-password-c"> Confirm Password </label>
      <input
        class="nitroTextInput"
        id="grid-password-c"
        type="password"
        v-model="confirm"
      />
    </div>
  </div>
</template>
<script lang="ts">
import { defineComponent, ref, watch } from "vue";

export default defineComponent({
  name: "PasswordBox",
  props: {
    modelValue: {
      required: true,
      type: String,
    },
  },
  setup(props, { emit }) {
    const password = ref("");
    const confirm = ref("");
    watch(confirm, () => {
      if (password.value === confirm.value) {
        emit("update:modelValue", password.value);
      } else {
        emit("update:modelValue", "");
      }
    });
    return { password, confirm };
  },
});
</script>
