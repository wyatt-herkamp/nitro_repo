<template>
  <div class="hidden md:flex min-h-screen">
    <div class="flex flex-col w-56 bg-slate-800">
      <ul class="flex flex-col py-4">
        
        <Item v-if="back != undefined" :href="'/admin/'+back" icon="arrow-back" name="Back" :active="back != undefined" />
        <Item href="/admin/users" icon="user" name="Users" :active="currentPage=='users'" />
        <Item href="/admin/storages" icon="box" name="Storages" :active="currentPage=='storages'" />
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
import UpdateUser from "@/components/UpdateUser.vue";

export default defineComponent({
  components: { Storages, Repositories, Users, UpdateUser, Me, Item },
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
});
</script>

