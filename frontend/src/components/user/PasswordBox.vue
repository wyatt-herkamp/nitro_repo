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
      <div>
        <input
          class="nitroTextInput"
          id="grid-password-c"
          type="password"
          v-model="confirm"
          ref="confirmBox"
        />

        <font-awesome-icon
          v-if="confirmIcon === 'password'"
          class="icon"
          icon="fa-solid fa-eye-slash"
          @click="switchType()"
        />
        <font-awesome-icon
          v-else-if="confirmIcon === 'text'"
          class="icon"
          icon="fa-solid fa-eye"
          @click="switchType()"
        />
      </div>
    </div>
  </div>
</template>
<script lang="ts">
import { computed, defineComponent, ref, watch } from "vue";

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
    const confirmBox = ref<HTMLInputElement>();
    const confirmIcon = ref("password");

    return { password, confirm, confirmBox, confirmIcon };
  },
  methods: {
    switchType() {
      if (this.confirmBox?.type == "password") {
        this.confirmBox?.setAttribute("type", "text");
        this.confirmIcon = "text";
      } else {
        this.confirmBox?.setAttribute("type", "password");
        this.confirmIcon = "password";
      }
    },
  },
});
</script>

<style>
.icon {
  margin-left: -30px;
  @apply cursor-pointer;
  @apply mt-2;
}
</style>
