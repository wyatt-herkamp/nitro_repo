<template>
  <RouterLink
    :to="to"
    :data-active="isActive"
    class="navLink">
    <slot />
  </RouterLink>
</template>
<script setup lang="ts">
import { computed, defineProps } from "vue";
import { useRouter } from "vue-router";
const props = defineProps({
  to: {
    type: String,
    required: true,
  },
  routeName: {
    type: String,
    required: false,
  },
});
const router = useRouter();
const isActive = computed(() => {
  return props.routeName === router.currentRoute.value.name;
});
</script>
<style scoped lang="scss">
@import "@/assets/styles/theme.scss";
.navLink {
  text-decoration: none;
  color: $text;
  font-weight: bold;
  padding: 0.5rem;
  // Align text vertically
  display: flex;
  align-items: center;
  gap: 0.5rem;
  // Box
  border-radius: 0.5rem;
  &:hover {
    background-color: $primary-70;
    transition: background-color 0.3s ease;
  }
}
.navLink[data-active="true"] {
  background-color: $primary-70;
  &:hover {
    cursor: default;
  }
}
</style>
