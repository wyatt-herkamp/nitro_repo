<template>
  <ul
    v-auto-animate
    class="proxyConfig">
    <li
      class="proxyRoute"
      v-for="route in value?.routes"
      :key="route.url">
      <input v-model="route.url" />
      <input v-model="route.name" />
      <input
        v-model="route.username"
        placeholder="Username (optional)" />
      <input
        v-model="route.password"
        type="password"
        placeholder="Password (optional)" />
      <button
        class="actionButton"
        @click="removeRoute(route)">
        Remove
      </button>
    </li>
    <li class="proxyRoute add">
      <input
        v-model="newRoute.url"
        placeholder="https://repo1.maven.org/maven2/" />
      <input
        v-model="newRoute.name"
        placeholder="Maven Central" />
      <input
        v-model="newRoute.username"
        placeholder="Username (optional)" />
      <input
        v-model="newRoute.password"
        type="password"
        placeholder="Password (optional)" />
      <button
        class="actionButton"
        @click="addRoute">
        Add
      </button>
    </li>
  </ul>
  <div class="ttl">
    <label>
      Artifact cache lifetime (seconds, 0 = forever)
      <input
        v-model.number="value.cache_ttl_seconds"
        type="number"
        min="0" />
    </label>
    <label>
      Metadata and snapshot lifetime (seconds)
      <input
        v-model.number="value.mutable_ttl_seconds"
        type="number"
        min="0" />
    </label>
    <p class="note">
      Released artifacts are immutable, so they are normally kept forever.
      <code>maven-metadata.xml</code> and snapshot builds change upstream and need a shorter
      lifetime, or a new release never shows up.
    </p>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { type MavenProxyRoute, type MavenProxyConfigType } from "./maven";
import { notify } from "@kyvg/vue3-notification";
const newRoute = ref<MavenProxyRoute>({
  url: "",
  name: "",
  username: "",
  password: "",
});

const value = defineModel<MavenProxyConfigType>({
  required: true,
});
function removeRoute(route: MavenProxyRoute) {
  value.value.routes = value.value.routes.filter((r) => r !== route);
}
function addRoute() {
  try {
    new URL(newRoute.value.url);
  } catch (e) {
    console.error("Invalid URL", e);
    notify({
      type: "error",
      title: "Invalid URL",
      text: "Please enter a valid URL",
    });
    return;
  }

  value.value.routes.push({
    url: newRoute.value.url,
    name: newRoute.value.name,
    // Empty strings would be sent as credentials and turned into a `Basic` header with a blank
    // username, which some upstreams reject outright.
    username: newRoute.value.username || undefined,
    password: newRoute.value.password || undefined,
  });
  newRoute.value.url = "";
  newRoute.value.name = "";
  newRoute.value.username = "";
  newRoute.value.password = "";
}
</script>

<style lang="scss" scoped>
@import "@/assets/styles/theme.scss";
.proxyRoute {
  display: flex;
  margin: 0.5rem;
  input {
    margin-right: 0.5rem;
  }
}
.actionButton {
  margin-left: 0.5rem;
}
.proxyConfig {
  list-style-type: none;
  padding: 0;
}
.ttl {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  margin: 0.5rem;
  label {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }
  .note {
    opacity: 0.8;
    font-size: 0.9rem;
  }
}
</style>
