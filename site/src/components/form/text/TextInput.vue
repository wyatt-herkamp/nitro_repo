<template>
  <section :id="id + '-section'">
    <label :for="id">
      <slot />
    </label>
    <span
      class="error"
      v-if="error"
      >{{ error }}</span
    >
    <div
      v-if="haveClearButton"
      class="input-container">
      <input
        type="text"
        :id="haveClearButton ? id : undefined"
        v-model="value"
        v-bind="$attrs" />
      <button>
        <font-awesome-icon
          v-if="value"
          @click="value = ''"
          icon="x" />
      </button>
    </div>
    <input
      v-else
      type="text"
      :id="haveClearButton ? undefined : id"
      v-model="value"
      v-bind="$attrs" />
  </section>
</template>
<script setup lang="ts">
import "@/assets/styles/form.scss";
import { FontAwesomeIcon } from "@fortawesome/vue-fontawesome";
defineProps({
  id: String,
  haveClearButton: {
    type: Boolean,
    default: false,
  },
  error: {
    type: String,
    required: false,
  },
});
const value = defineModel<string | undefined>({
  required: true,
});
</script>

<style scoped lang="scss">
@import "@/assets/styles/theme.scss";
@import "@/assets/styles/form.scss";

.input-container {
  position: relative;
  display: inline-block;
  width: 100%;

  input {
    width: 100%;
  }

  button {
    position: absolute;
    right: 10px;
    top: 50%;
    transform: translateY(-25%);
    border: none;
    background: transparent;
    cursor: pointer;
    font-size: 16px;
    color: $text;
    transition: color 0.3s ease;
  }
}
</style>
