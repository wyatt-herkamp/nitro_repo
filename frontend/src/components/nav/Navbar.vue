<template>
  <!-- Navbar goes here -->
  <nav class="shadow-lg bg-secondary">
    <div class="max-w-6xl mx-auto px-4">
      <div class="flex justify-between">
        <div class="flex space-x-7">
          <div>
            <!-- Website Logo -->
            <router-link to="/" class="flex items-center py-4 px-2">
              <img src="/icon-128.png" alt="Logo" class="h-8 w-8 mr-2" />
              <span class="font-semibold text-white text-lg"
                >Nitro Repository</span
              >
            </router-link>
          </div>

          <ul class="hidden mediumMenu">
            <li class="fullScreenItem">
              <router-link to="/">Home</router-link>
            </li>
            <li class="fullScreenItem">
              <router-link to="/browse">Browse</router-link>
            </li>
          </ul>
        </div>

        <!-- Secondary Navbar items -->
        <ul class="hidden md:flex items-center space-x-3">
          <li v-if="user === undefined" class="fullScreenItem">
            <button @click="openLogin = true">Login</button>
          </li>
          <li v-if="user !== undefined" class="fullScreenItem">
            <router-link to="/admin">Admin</router-link>
          </li>
          <li v-if="user !== undefined" class="fullScreenItem">
            <router-link to="/me">Me</router-link>
          </li>
        </ul>
        <!-- Mobile menu button -->
        <div class="md:hidden flex items-center">
          <button @click="openNav" class="outline-none mobile-menu-button">
            <svg
              class="w-6 h-6 text-tertiary hover:text-accent"
              fill="none"
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path d="M4 6h16M4 12h16M4 18h16"></path>
            </svg>
          </button>
        </div>
      </div>
    </div>
  </nav>
  <div ref="mobileNav" class="hidden smMenu">
    <ul class="flex flex-col w-full">
      <li class="smItem">
        <router-link to="/" @click="openNav" class="smItem">Home</router-link>
      </li>
      <li class="smItem">
        <router-link to="/browse" @click="openNav" class="smItem"
          >Browse</router-link
        >
      </li>
      <li v-if="user !== undefined" class="smItem">
        <router-link to="/me" @click="openNav" class="smItem">Me</router-link>
      </li>
      <li class="smItem">
        <router-link v-if="user === undefined" to="/login" @click="openNav"
          >Login</router-link
        >
      </li>

      <AdminDropBox @clicked="openNav()" v-if="user !== undefined" />
    </ul>
  </div>
  <Login v-model="openLogin" />
</template>

<script lang="ts">
import { defineComponent, ref } from "vue";
import { useRouter } from "vue-router";
import Login from "./Login.vue";
import AdminDropBox from "./AdminDropBox.vue";
import { User } from "@/types/user";

export default defineComponent({
  props: {
    user: {
      required: false,
      type: Object as () => User,
    },
  },
  components: { Login, AdminDropBox },
  setup() {
    const router = useRouter();
    const activeIndex = ref(router.currentRoute.value.name);
    const mobileNav = ref<HTMLDivElement>();
    const navOpen = ref(false);
    const openLogin = ref(false);
    const openNav = (): void => {
      navOpen.value = !navOpen.value;

      if (navOpen.value) {
        mobileNav.value?.classList.remove("hidden");
        mobileNav.value?.classList.add("flex");
      } else {
        mobileNav.value?.classList.add("hidden");
        mobileNav.value?.classList.remove("flex");
      }
    };

    return {
      activeIndex,
      router,
      openNav,
      mobileNav,
      openLogin,
    };
  },
});
</script>
<style scoped>
.fullScreenItem {
  @apply text-quaternary;
  @apply px-2;
  @apply hover:bg-primary/10;
  @apply tracking-wide;
}

.mediumMenu {
  @apply md:items-center;
  @apply md:space-x-1;
  @apply md:flex;
  @apply md:flex-row;
}
.smMenu {
  @apply border-t-2;
  @apply border-primary/10;
  @apply w-full;
  @apply bg-secondary/90;
  @apply shadow-md;
  @apply rounded-b-md;
}

.smItem {
  @apply text-quaternary;
  @apply w-full;
  @apply block;
  @apply tracking-wide;
  @apply py-2;
  @apply hover:bg-secondary/80;
  @apply text-center;
}
</style>
