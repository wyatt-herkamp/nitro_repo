<template>
  <div class="w-full">
    <SimpleTabs :defaultTab="page" @tabChange="onTabChange">
      <SimpleTab name="General">
        <div class="w-1/2 mx-auto bg-tertiary mt-5 rounded-l">
          <h1 class="text-quaternary text-2xl mx-2 mt-4 border-b-2 w-fit px-2">
            Installed Version Info
          </h1>
          <table class="table-auto text-quaternary">

            <tbody>
              <tr v-for="(value, key) in version" v-bind:key="key">
                <th scope="row">{{ key }}</th>
                <td>{{ value }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </SimpleTab>
      <SimpleTab name="Users">
        <Users />
      </SimpleTab>
      <SimpleTab name="Storages">
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
import { useSystemStore } from "@/store/system";

export default defineComponent({
  components: {
    Storages,
    Users,
  },

  setup() {
    const systemStore = useSystemStore();
    const version = computed(() => {
      return systemStore.$state.version;
    });
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
      version,
    };
  },
  methods: {
    onTabChange(tab: string) {
      this.$router.replace("/admin/" + tab);
    },
  },
});
</script>
<style>
table {
  border-collapse: collapse;
  width: 100%;
}

th,
td {
  text-align: left;
  padding: 8px;
}
tr:nth-child(even) {
  @apply bg-tertiary/50;
}
</style>
