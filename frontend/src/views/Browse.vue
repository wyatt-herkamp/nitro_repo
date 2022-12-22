<template>
  <div id="browseBox">
    <div
      :class="[activeResponse !== undefined ? 'lg:basis-1/2' : 'flex-grow']"
      class="m-2 rounded-md bg-gray-900"
    >
      <div class="flex flex-col">
        <div>
          <div class="m-5">
            <router-link class="backLink text-quaternary" to="/browse">
              <span>Browse</span>
            </router-link>
            <router-link
              class="backLink text-quaternary mx-1 sm:m-0 inline-block sm:inline"
              v-for="value in pathSplit"
              :key="value.name"
              :to="'/browse' + value.path"
            >
              <span>/</span>
              <span> {{ value.name }} </span>
            </router-link>
          </div>
        </div>
        <Suspense fallback="Loading...">
          <ListInsideRepository
            v-if="storage !== '' && repository !== ''"
            :storage="storage"
            :repository="repository"
            :catchAll="catchAll"
            v-model="pathSplit"
          />
          <ListRepositories
            v-else-if="storage !== ''"
            :storage="storage"
            v-model="pathSplit"
          />
          <ListStorages v-else v-model="pathSplit" />
        </Suspense>
      </div>
    </div>
    <!-- Optional Extra Info -->
  </div>
</template>

<script lang="ts">
import { apiURL } from "@/http-common";
import { defineComponent, ref } from "vue";
import { useRoute } from "vue-router";
import { BrowsePath } from "@/api/Browse";
import { ResponseType } from "@/types/repositoryTypes";
import ListInsideRepository from "@/components/browse/ListInsideRepository.vue";
import ListRepositories from "@/components/browse/ListRepositories.vue";
import ListStorages from "@/components/browse/ListStorages.vue";
import "@/styles/browse.scss";

export default defineComponent({
  setup() {
    const url = apiURL;
    const route = useRoute();
    const pathSplit = ref<BrowsePath[]>([]);
    const activeResponse = ref<ResponseType | undefined>();
    const storage = ref<string | undefined>(route.params.storage as string);
    const repository = ref<string | undefined>(route.params.repo as string);
    const catchAll = ref(route.params.catchAll as string);

    return {
      storage,
      repository,
      catchAll,
      pathSplit,
      url,
      activeResponse,
    };
  },
  components: {
    ListStorages,
    ListRepositories,
    ListInsideRepository,
  },
});
</script>
