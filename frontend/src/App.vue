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
import { useMeta } from "vue-meta";

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
<style>
#app {
  font-family: Avenir, Helvetica, Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  text-align: center;
  color: white;
}

#nav {
  padding: 30px;
}

#nav a {
  font-weight: bold;
  color: #2c3e50;
}

#nav a.router-link-exact-active {
  color: #42b983;
}
</style>
