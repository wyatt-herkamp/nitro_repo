<template>
  <div id="nav">
    <router-link to="/">Home</router-link> |
    <router-link to="/about">About</router-link>
  </div>

  <p>
    {{ defaultStore.state.installed }}
    {{ userStore.state.user.username }}
  </p>
  <router-view />
</template>

<script lang="ts">
import userStore from "@/store/user";
import defaultStore from "@/store/default";
import router from "@/router";
import { defineComponent, onMounted } from "vue";
export default defineComponent({
  name: "App",
  components: {},
  setup() {
    onMounted(defaultStore.init);
    onMounted(userStore.getUser);
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
