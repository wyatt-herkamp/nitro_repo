<template>
  <nav class="navBar">
    <div class="navInner">
      <RouterLink
        to="/"
        class="brand">
        <img
          src="/icon-128.png"
          alt=""
          width="24"
          height="24" />
        <span class="brandName">Nitro Repo</span>
      </RouterLink>

      <div class="navLinks">
        <RouterLink
          :to="{ name: 'repositories' }"
          class="navLink"
          >Repositories</RouterLink
        >
        <RouterLink
          :to="{ name: 'search' }"
          class="navLink"
          >Search</RouterLink
        >
        <RouterLink
          v-if="canAdmin"
          :to="{ name: 'admin' }"
          class="navLink"
          >Admin</RouterLink
        >
      </div>

      <div class="navRight">
        <button
          type="button"
          class="themeButton"
          :title="`Theme: ${preference}`"
          :aria-label="`Theme: ${preference}. Click to change.`"
          @click="cycle">
          <font-awesome-icon :icon="themeIcon" />
        </button>

        <UserDropDown v-if="user" />
        <RouterLink
          v-else
          :to="{ name: 'login' }"
          class="navLink isLogin"
          >Sign in</RouterLink
        >
      </div>
    </div>
  </nav>
</template>

<script setup lang="ts">
import { computed, type PropType } from "vue";
import UserDropDown from "./UserDropDown.vue";
import type { UserResponseType } from "@/types/base";
import { useTheme } from "@/composables/theme";

const props = defineProps({
  user: {
    type: Object as PropType<UserResponseType>,
    required: false,
  },
});

const { preference, cycle } = useTheme();

const themeIcon = computed(() =>
  preference.value === "dark" ? "moon" : preference.value === "light" ? "sun" : "desktop",
);

// The admin area is refused server-side regardless; this only stops the bar offering a link that
// would bounce straight back.
const canAdmin = computed(
  () => props.user?.admin || props.user?.user_manager || props.user?.system_manager,
);
</script>

<style scoped lang="scss">
.navBar {
  position: sticky;
  top: 0;
  z-index: var(--z-nav);
  height: var(--nav-height);
  background-color: var(--surface);
  border-bottom: 1px solid var(--border);
  // The bar is the one place with any atmosphere: a hairline of the accent along the top edge, so
  // the chrome reads as an instrument frame rather than a plain band.
  box-shadow: inset 0 1px 0 var(--accent-muted);
}

.navInner {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  height: 100%;
  max-width: var(--container);
  margin-inline: auto;
  padding-inline: var(--space-6);

  @media (max-width: 40rem) {
    padding-inline: var(--space-3);
  }
}

.brand {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  margin-right: var(--space-4);
  color: var(--text);
  font-weight: var(--weight-semibold);
  letter-spacing: -0.01em;
  white-space: nowrap;

  &:hover {
    color: var(--text);
  }

  img {
    border-radius: var(--radius-sm);
  }
}

.navLinks {
  display: flex;
  align-items: center;
  gap: var(--space-1);
}

.navLink {
  padding: 0.375rem 0.625rem;
  font-size: var(--text-sm);
  font-weight: var(--weight-medium);
  color: var(--text-muted);
  border-radius: var(--radius-md);
  white-space: nowrap;

  &:hover {
    color: var(--text);
    background-color: var(--surface-hover);
  }

  // `router-link-active` matches parent paths too, so the exact class is what should light up.
  &.router-link-exact-active {
    color: var(--text);
    background-color: var(--surface-active);
  }

  &.isLogin {
    color: var(--accent);
  }
}

.navRight {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  margin-left: auto;
}

.themeButton {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 1.875rem;
  height: 1.875rem;
  color: var(--text-muted);
  background: transparent;
  border: 1px solid transparent;
  border-radius: var(--radius-md);
  cursor: pointer;
  transition:
    color var(--duration-fast) var(--ease-out),
    background-color var(--duration-fast) var(--ease-out);

  &:hover {
    color: var(--text);
    background-color: var(--surface-hover);
  }
}

@media (max-width: 34rem) {
  .brandName {
    display: none;
  }
}
</style>
