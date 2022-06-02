<template>
  <div class="flex flex-col min-h-screen">
    <Navbar :user="user" />
    <router-view :key="$route.fullPath" />
  </div>

  <notifications position="bottom right" />
</template>

<script lang="ts">
import Navbar from "@/components/nav/Navbar.vue";
import { computed, defineComponent, onMounted } from "vue";
import useUserStore from "@/store/user";

export default defineComponent({
  name: "App",
  components: { Navbar },
  setup() {
    const userStore = useUserStore();
    onMounted(userStore.getAccount);
    const user = computed(() => userStore.$state.user);
    return { user };
  },
});
</script>
