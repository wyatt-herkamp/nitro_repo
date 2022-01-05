<template></template>

<script lang="ts">
import {defineComponent, ref} from "vue";
import CreateStorage from "@/components/CreateStorage.vue";
import UpdateStorage from "@/components/UpdateStorage.vue";
import {useCookie} from "vue-cookie-next";
import {getStorages} from "@/backend/api/Storages";
import {DEFAULT_STORAGE_LIST} from "@/backend/Response";

export default defineComponent({
  components: { CreateStorage, UpdateStorage },

  setup() {
    let index = ref(1);
    const cookie = useCookie();
    const isLoading = ref(false);

    const error = ref("");
    let storages = ref(DEFAULT_STORAGE_LIST);
    const getStorage = async () => {
      isLoading.value = true;
      try {
        const value = await getStorages(cookie.getCookie("token"));
        storages.value = value;

        isLoading.value = false;
      } catch (e) {
        error.value = "Error";
      }
    };
    getStorage();
    return {
      index,
      storages,
      isLoading,
      error,
      getStorage,
      cookie,
    };
  },
  methods: {
    updateList(id: number) {
      const getStorage = async () => {
        try {
          const value = await getStorages(this.cookie.getCookie("token"));
          this.storages = value;
          this.index = id;
        } catch (e) {
          this.error = "Error";
        }
      };
      getStorage();
    },
  },
});
</script>

<style>
.el-menu-vertical-demo:not(.el-menu--collapse) {
  width: 200px;
  min-height: 400px;
}
</style>
