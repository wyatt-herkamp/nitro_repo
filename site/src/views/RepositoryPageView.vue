<template>
  <main v-if="repository" :data-has-page="repositoryPage != undefined">
    <div class="header">
      <h1>{{ repository.storage_name }}/{{ repository.name }}</h1>
      <div class="openBrowse">
        <RouterLink
          :to="{
            name: 'Browse',
            params: { id: repository.id, catchAll: '' }
          }"
          >Browse</RouterLink
        >
      </div>
      <div>
        Maven
        <ApacheMavenIcon />
      </div>
    </div>
    <div class="content">
      <div id="page">
        <RepositoryPageViewer
          v-if="repositoryPage"
          :repository="repository"
          :page="repositoryPage" />
      </div>
      <div id="helper">
        <RepositoryHelper :repository="repository" />
      </div>
    </div>
  </main>
  <ErrorOnRequest v-else-if="error" :error="error" :errorCode="errorCode" />
</template>

<script setup lang="ts">
import AdminUserPage from '@/components/admin/user/AdminUserPage.vue'
import ErrorOnRequest from '@/components/ErrorOnRequest.vue'
import RepositoryHelper from '@/components/repository/RepositoryHelper.vue'
import RepositoryPageViewer from '@/components/repository/RepositoryPageViewer.vue'
import http from '@/http'
import { ApacheMavenIcon, NpmIcon } from 'vue3-simple-icons'

import router from '@/router'
import { repositoriesStore } from '@/stores/repositories'
import type { User } from '@/types/base'
import type { RepositoryPage, RepositoryWithStorageName } from '@/types/repository'
import { computed, ref } from 'vue'
const repositoryId = router.currentRoute.value.params.id as string
const repository = ref<RepositoryWithStorageName | undefined>(undefined)
const repositoryPage = ref<RepositoryPage | undefined>(undefined)
const error = ref<string | null>(null)
const errorCode = ref<number | undefined>(undefined)
const repoStore = repositoriesStore()

async function fetchRepository() {
  await repoStore.getRepositoryById(repositoryId).then((response) => {
    repository.value = response
    console.log(repository.value)
  })
  await http
    .get<RepositoryPage>(`/api/repository/page/${repositoryId}`)
    .then((response) => {
      console.log(response.data)
      repositoryPage.value = response.data
    })
    .catch((error) => {
      console.error(error)
      errorCode.value = error.response.status
      error.value = 'Failed to fetch repository'
    })
}
fetchRepository()
</script>
<style scoped lang="scss">
@import '@/assets/styles/theme.scss';
main {
  margin: 0 auto;
  padding: 1rem;
}
.header {
  display: flex;
  flex-direction: row;
  align-items: center;
  justify-content: space-between;
  border-bottom: 1px solid gray;

  h1 {
    margin: 0;
  }
}
main[data-has-page='true'] {
  width: 75%;
  .content {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    flex-wrap: wrap;
    gap: 1rem;
    justify-content: space-between;
  }
}
#page {
  flex-grow: 3;
  width: 100%;
  border-bottom: 1px solid gray;
}
@media screen and (max-width: 1200px) {
  main[data-has-page='true'] {
    width: 100%;
  }
  #page {
    border-bottom: none;
  }
}
.openBrowse {
  a {
    text-decoration: none;
    color: $text;
  }
}
</style>
