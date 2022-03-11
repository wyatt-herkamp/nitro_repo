<template>
  <div class="min-h-screen flex">
    <div class="flex flex-col w-56 bg-slate-800">
      <ul class="flex flex-col py-4">
        
        <Item v-if="back != undefined" :href="'/admin/'+back" icon="arrow-back" name="Back" :active="back != undefined" />
        <Item href="/admin/users" icon="user" name="Users" :active="currentPage=='users'" />
        <Item href="/admin/repositories" icon="package" name="Repositories" :active="currentPage=='repositories'" />
        <Item href="/admin/storages" icon="box" name="Storages" :active="currentPage=='storages'" />
        <Item
          href="/admin/settings"
          icon="dots-horizontal-rounded"
          name="Settings"
           :active="currentPage=='settings'" 
        />
      </ul>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, onBeforeMount, ref } from "vue";
import Storages from "@/components/Storages.vue";
import Users from "@/components/Users.vue";
import Me from "@/components/Me.vue";
import Repositories from "@/components/Repositories.vue";
import Item from "@/components/admin/Item.vue";
import Settings from "@/components/Settings.vue";
import UpdateUser from "@/components/UpdateUser.vue";
import userStore from "@/store/user";

export default defineComponent({
  components: { Storages, Repositories, Users, UpdateUser, Settings, Me, Item },
  props: {
    back: {
      required: false,
      type: String,
    },    
    currentPage: {
      required: false,
      type: String,
    },
  },
  setup() {
    let index = ref(4);
    onBeforeMount(userStore.getUser);
    return {
      index,
      userStore,
    };
  },
});
</script>

