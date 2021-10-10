<template>
  <Navbar :user="userStore.state.user" />

  <router-view />
  <notifications position="bottom right" />

</template>

<script lang="ts">
import userStore from "@/store/user";
import defaultStore from "@/store/default";
import Navbar from "@/components/Navbar.vue";
import router from "@/router";
import { defineComponent, onBeforeMount, onMounted } from "vue";
export default defineComponent({
  name: "App",
  components: { Navbar },
  setup() {
    onBeforeMount(defaultStore.init);
    onBeforeMount(userStore.getUser);
    return { userStore, defaultStore };
  },
});
</script>
<style>
#app {
  font-family: Avenir, Helvetica, Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  text-align: center;
  color: #2c3e50;
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
