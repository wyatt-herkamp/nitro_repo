<template>
  <!-- Navbar goes here -->
  <nav class="bg-slate-900 shadow-lg">
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
            <li>
              <router-link to="/" class="fullScreenItem">Home</router-link>
            </li>
            <li>
              <router-link to="/browse" class="fullScreenItem"
                >Browse</router-link
              >
            </li>
          </ul>
        </div>

        <!-- Secondary Navbar items -->
        <ul class="hidden md:flex items-center space-x-3">
          <button
            v-if="user == undefined"
            @click="openLogin = true"
            class="fullScreenItem login"
          >
            Login
          </button>
          <li v-if="user != undefined">
            <router-link to="/admin" class="fullScreenItem login"
              >Admin</router-link
            >
          </li>
        </ul>
        <!-- Mobile menu button -->
        <div class="md:hidden flex items-center">
          <button @click="openNav" class="outline-none mobile-menu-button">
            <svg
              class="w-6 h-6 text-gray-500 hover:text-green-500"
              x-show="!showMenu"
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
    <div class="w-44 rounded-md float-right bg-slate-800">
      <ul class="flex flex-col py-4">
        <li>
          <router-link to="/" @click="openNav" class="smItem">Home</router-link>
        </li>
        <li>
          <router-link to="/browse" @click="openNav" class="smItem"
            >Browse</router-link
          >
        </li>
        <li>
          <router-link to="/login" @click="openNav" class="smItem login"
            >Login</router-link
          >
        </li>

        <AdminDropBox @clicked="openNav()" v-if="user != undefined" />
      </ul>
    </div>
  </div>
  <Login v-model="openLogin" />
</template>

<script lang="ts">
import { defineComponent, reactive, ref } from "vue";
import { useRouter } from "vue-router";
import { User } from "nitro_repo-api-wrapper";
import MenuButton from "@/components/nav/MenuButton.vue";
import Login from "@/components/nav/Login.vue";
import { react } from "@babel/types";
import AdminDropBox from "./AdminDropBox.vue";

export default defineComponent({
  props: {
    user: {
      required: false,
      type: Object as () => User,
    },
  },
  components: { MenuButton, Login, AdminDropBox },
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
  @apply inline;
  @apply py-4;
  @apply px-2;
  @apply font-semibold;
  @apply hover:text-slate-300;
  @apply transition;
  @apply duration-300;
}
.login {
  @apply font-medium;
  @apply text-white;
  @apply bg-slate-800;
  @apply rounded;
  @apply hover:bg-slate-900;
}
.mediumMenu {
  @apply md:items-center;
  @apply md:space-x-1;
  @apply md:flex;
  @apply md:flex-row;
}
.smMenu {
  @apply absolute;
  @apply right-0;
  @apply h-auto;
  min-height: 25%;
  @apply transition;
  @apply ease-in-out;
  @apply duration-300;
}

.smItem {
  @apply block;
  @apply text-sm;
  @apply px-4;
  @apply pt-4;
  @apply pb-6;
  @apply hover:bg-green-500;
  @apply transition;
  @apply duration-300;
  @apply text-left;
}
</style>
