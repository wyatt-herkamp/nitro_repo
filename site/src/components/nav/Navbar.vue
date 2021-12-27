<template>
  <nav class="flex flex-wrap bg-slate-900 p-6">
    <router-link to="/">
      <MenuButton> Nitro Repo</MenuButton>
    </router-link>
    <router-link to="/browse">
      <MenuButton> Browse</MenuButton>
    </router-link>
    <router-link v-if="user.id != 0" to="/admin">
      <MenuButton> Admin</MenuButton>
    </router-link>
    <router-link v-if="user.id != 0" class="end" to="/me">
      <MenuButton class="end"> Welcome, {{ user.name }}</MenuButton>
    </router-link>

    <Login v-if="user.id == 0" class="end">
      <template v-slot:button>
        <MenuButton> Sign in</MenuButton>
      </template>
    </Login>
  </nav>
</template>

<script lang="ts">
import {defineComponent, ref} from "vue";
import {useRouter} from "vue-router";
import {User} from "@/backend/Response";
import MenuButton from "@/components/nav/MenuButton.vue";
import Login from "@/components/nav/Login.vue";

export default defineComponent({
  props: {
    user: {
      required: true,
      type: Object as () => User,
    },
  },
  components: {MenuButton, Login},
  setup() {
    const router = useRouter();
    const activeIndex = ref(router.currentRoute.value.name);
    return {activeIndex, router};
  },
});
</script>
<style scoped>
.end {
  margin-left: auto;
}
</style>