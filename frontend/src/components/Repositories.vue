<template>
  <div class="w-full">
    <div class="flex p-4">
      <div class="w-full float-left">
        <div class="bg-slate-800 shadow-md rounded-lg px-3 py-2 mb-4">
          <div class="block text-slate-50 text-lg font-semibold py-2 px-2">
            Repositories
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
                placeholder="Repository Name"
                type="text"
              />
            </div>
            <CreateRepo :storage="storage"> </CreateRepo>
          </div>
          <div>
            <ul v-if="repositories != undefined">
              <li v-for="repo in repositories.repositories" :key="repo.name">
                <router-link
                  :to="'/admin/repository/' + repo.storage + '/' + repo.name"
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
                  <div class="px-1">{{ repo.name }}</div>
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
import CreateRepo from "@/components/CreateRepo.vue";
import { useCookie } from "vue-cookie-next";
import { getRepositoriesByStorage } from "nitro_repo-api-wrapper";
import { RepositoryList, RepositoryListResponse } from "nitro_repo-api-wrapper";
import { getRepositories } from "nitro_repo-api-wrapper";

export default defineComponent({
  components: { CreateRepo },
  props: {
    storage: {
      type: Object as () => Storage,
      required: true,
    },
  },
  setup(props) {
    const index = ref<number>(0);
    const isLoading = ref(false);
    const cookie = useCookie();

    const error = ref("");
    let repositories = ref<RepositoryList | undefined>(undefined);
    const getRepos = async () => {
      isLoading.value = true;
      try {
        const value = await getRepositoriesByStorage(
          cookie.getCookie("token"),
          props.storage.name
        );
        console.log(value);
        repositories.value = value;
        isLoading.value = false;
      } catch (e) {
        error.value = "Error";
      }
    };
    getRepos();
    return {
      cookie,
      index,
      repositories,
      isLoading,
      error,
      getRepos,
    };
  },
  methods: {
    updateList(id: number) {
      console.log("Updating Repos");
      const getRepos = async () => {
        try {
          const value = await getRepositories(this.cookie.getCookie("token"));
          this.repositories = value;
          this.index = id;
        } catch (e) {
          this.error = "Error";
        }
      };
      getRepos();
    },
  },
});
</script>
