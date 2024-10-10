<template>
  <div class="copyURL">
    <label>
      <slot></slot>
    </label>
    <span @click="copyURL">
      {{ code }}
    </span>
  </div>
</template>
<script setup lang="ts">
import { notify } from "@kyvg/vue3-notification";

const props = defineProps({
  code: {
    type: String,
    required: true,
  },
});
function copyURL() {
  navigator.clipboard.writeText(props.code);
  notify({
    type: "success",
    title: "Copied",
  });
}
</script>

<style lang="scss" scoped>
@import "@/assets/styles/theme.scss";
.copyURL {
  margin: 1rem;
}
@media screen and (max-width: 768px) {
  span {
    max-width: 90%;
    word-wrap: break-word;
  }
}
span {
  display: block;
  width: fit-content;
  cursor: pointer;
  padding: 0.5rem;
  padding-right: 1rem;
  border-radius: 0.25rem;
  margin-top: 0.5rem;
  border: 0.25px solid $primary-50;
  // Text wrapping

  &:hover {
    background-color: $primary-50;
    transition: background-color 0.25s;
  }
}
</style>
