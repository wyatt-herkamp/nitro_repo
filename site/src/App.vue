<template>
  <Navbar :user="userStore.state.user" />
  <metainfo> </metainfo>
  <router-view :key="$route.fullPath" />
  <notifications position="bottom right" />
</template>

<script lang="ts">
import userStore from "@/store/user";
import Navbar from "@/components/nav/Navbar.vue";
import {defineComponent, onBeforeMount} from "vue";
import {useMeta} from "vue-meta";

export default defineComponent({
  name: "App",
  components: { Navbar },
  setup() {
    onBeforeMount(userStore.getUser);
    return { userStore };
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
