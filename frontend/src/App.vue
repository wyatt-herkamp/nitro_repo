<template>
  <metainfo> </metainfo>
  <div class="flex flex-col min-h-screen">
    <Navbar :user="user" />
    <router-view :key="$route.fullPath" />
  </div>

  <notifications position="bottom right" />
</template>

<script lang="ts">
import { useUserStore } from "@/store/user";
import Navbar from "@/components/nav/Navbar.vue";
import { computed, defineComponent, onMounted } from "vue";

export default defineComponent({
  name: "App",
  components: { Navbar },
  setup() {
    const userStore = useUserStore();
    onMounted(userStore.loadUser);
    const user = computed(() => {
      return userStore.$state.user;
    });
    return { user: user };
  },
});
</script>
