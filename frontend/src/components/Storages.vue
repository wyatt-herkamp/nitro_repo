<template>
  <div class="w-full">
    <div class="flex p-4">
      <div class="w-full float-left">
        <div class="bg-slate-800 shadow-md rounded-lg px-3 py-2 mb-4">
          <div class="block text-slate-50 text-lg font-semibold py-2 px-2">
            Storages
          </div>
          <div class="flex flex-row">
            <div class="flex items-center bg-gray-200 rounded-md w-full h-max">
              <div class="pl-2">
                <svg
                  class="fill-current text-gray-500 w-6 h-6"
                  viewBox="0 0 24 24"
                  xmlns="http://www.w3.org/2000/svg"
                >
                  <path
                    class="heroicon-ui"
                    d="M16.32 14.9l5.39 5.4a1 1 0 0 1-1.42 1.4l-5.38-5.38a8 8 0 1 1 1.41-1.41zM10 16a6 6 0 1 0 0-12 6 6 0 0 0 0 12z"
                  />
                </svg>
              </div>
              <input
                id="search"
                class="
                  w-full
                  rounded-md
                  bg-gray-200
                  text-gray-700
                  leading-tight
                  focus:outline-none
                  py-2
                  px-2
                "
                placeholder="Storage Name"
                type="text"
              />
            </div>
            <CreateStorage />
          </div>
          <div>
            <ul v-if="storages != undefined">
              <li v-for="storage in storages" :key="storage.name">
                <router-link
                  :to="'/admin/storage/' + storage.name"
                  class="
                    cursor-pointer
                    py-2
                    text-slate-50
                    flex flex-row
                    m-1
                    hover:translate-x-2
                    transition-transform
                    ease-in
                    duration-200
                  "
                >
                  <div class="px-1">{{ storage.name }}</div>
                </router-link>
              </li>
            </ul>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref } from "vue";
import CreateStorage from "@/components/CreateStorage.vue";
import UpdateStorage from "@/components/UpdateStorage.vue";
import { useCookie } from "vue-cookie-next";
import { getStorages } from "nitro_repo-api-wrapper";
import { StorageList } from "nitro_repo-api-wrapper";

export default defineComponent({
  components: { CreateStorage, UpdateStorage },

  setup() {
    let index = ref(1);
    const cookie = useCookie();
    const isLoading = ref(false);

    const error = ref("");
    let storages = ref<StorageList | undefined>();
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

