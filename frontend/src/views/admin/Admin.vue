<template>
  <div class="w-full">
    <Tabs :defaultTab="page" @tabChange="onTabChange">
      <Tab name="General" icon="tachometer">
        <h1>General</h1>
        <h3>Coming Soon</h3>
      </Tab>
      <Tab name="Users" icon="user">
        <Users />
      </Tab>
      <Tab name="Storages" icon="box">
        <Storages />
      </Tab>
    </Tabs>
  </div>
</template>

<script lang="ts">
import { computed, defineComponent, ref } from "vue";
import Storages from "@/components/Storages.vue";
import Users from "@/components/Users.vue";
import { useRoute } from "vue-router";
import Tabs from "@/components/common/tabs/Tabs.vue";
import Tab from "@/components/common/tabs/Tab.vue";
import { useUserStore } from "@/store/user";
import { useMeta } from "vue-meta";

export default defineComponent({
  components: {
    Storages,
    Users,
    Tabs,
    Tab,
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
      this.$router.push("/admin/" + tab);
    },
  },
});
</script>
