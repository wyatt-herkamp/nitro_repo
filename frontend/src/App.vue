<template>
  <metainfo> </metainfo>
  <div id="container">
    <Navbar :user="user" />
    <router-view :key="$route.fullPath" />
  </div>

  <notifications position="bottom right" />
</template>
<style scoped>
#container {
  height: 100vh;
}
</style>
<script lang="ts">
import { useUserStore } from "@/store/user";
import Navbar from "@/components/nav/Navbar.vue";
import { computed, defineComponent, onMounted } from "vue";
import { useSystemStore } from "@/store/system";

export default defineComponent({
  name: "App",
  components: { Navbar },
  setup() {
    const systemStore = useSystemStore();
    const userStore = useUserStore();
    onMounted(userStore.loadUser);
    onMounted(systemStore.load);
    const user = computed(() => {
      return userStore.$state.user;
    });
    const system = computed(() => {
      return systemStore.$state;
    });
    return { user, system };
  },
});
</script>
