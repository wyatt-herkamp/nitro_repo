<template>
  <Navbar :user="userStore.state.user" />
  <metainfo> </metainfo>
  <router-view :key="$route.fullPath" />
  <notifications position="bottom right" />
</template>

<script lang="ts">
import userStore from "@/store/user";
import siteStore from "@/store/site";
import Navbar from "@/components/nav/Navbar.vue";
import {defineComponent, onBeforeMount} from "vue";
import {useMeta} from "vue-meta";

export default defineComponent({
  name: "App",
  components: { Navbar },
  setup() {
    onBeforeMount(userStore.getUser);
    siteStore.getSiteInfo();

    useMeta({
      title: siteStore.state.name,
      description: siteStore.state.description,
      htmlAttrs: {lang: "en", amp: false, charset: "UTF-8"},
      meta: [
        {
          property: "og:title",
          content: siteStore.state.name,
        },
        {
          property: "og:description",
          content: siteStore.state.description,
        },
        {property: "og:type", content: "website"},
      ],
    });
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
