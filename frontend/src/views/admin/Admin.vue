<template>
  <div class="w-full">
    <SimpleTabs :defaultTab="page" @tabChange="onTabChange">
      <SimpleTab name="General" icon="tachometer">
        <h1>General</h1>
        <h3>Coming Soon</h3>
      </SimpleTab>
      <SimpleTab name="Users" icon="user">
        <Users />
      </SimpleTab>
      <SimpleTab name="Storages" icon="box">
        <Storages />
      </SimpleTab>
    </SimpleTabs>
  </div>
</template>

<script lang="ts">
import { computed, defineComponent, ref } from "vue";
import Storages from "@/components/Storages.vue";
import Users from "@/components/Users.vue";
import { useRoute } from "vue-router";
import { useUserStore } from "@/store/user";
import { useMeta } from "vue-meta";

export default defineComponent({
  components: {
    Storages,
    Users,
  },

  setup() {
    const route = useRoute();
    const page = ref(
      route.params.page ? (route.params.page as string) : undefined
    );
    const userStore = useUserStore();
    const user = computed(() => {
      return userStore.$state.user;
    });
    useMeta({
      title: page.value ? page.value + " - Admin" : "Admin",
    });
    return {
      user,
      page,
    };
  },
  methods: {
    onTabChange(tab: string) {
      this.$router.replace("/admin/" + tab);
    },
  },
});
</script>
