<template>
  <div id="repositoryBox">
    <div id="headerBar">
      <h2>Repositories</h2>
      <input
        type="text"
        id="nameSearch"
        v-model="searchValue"
        autofocus
        placeholder="Search by Name, Storage Name" />
    </div>
    <div id="repositories" class="betterScroll">
      <div class="row" id="header">
        <div
          :class="['col', { sorted: sortBy === 'id' }]"
          @click="sortBy = 'id'"
          title="Sort by ID">
          ID #
        </div>
        <div
          :class="['col', { sorted: sortBy === 'name' }]"
          @click="sortBy = 'name'"
          title="Sort by Name">
          Name
        </div>
        <div
          :class="['col', { sorted: sortBy === 'storage-type' }]"
          @click="sortBy = 'storage-type'"
          title="Sort by Storage Type">
          Storage Name
        </div>
        <div :class="['col']">Repository Type</div>
        <div :class="['col']">Active</div>
      </div>
      <div
        class="row item"
        v-for="repository in filteredTable"
        :key="repository.id"
        @click="router.push({ name: 'repository', params: { id: repository.id } })">
        <div class="col">{{ repository.id }}</div>
        <div class="col" :title="repository.name">{{ repository.name }}</div>
        <div class="col" :title="repository.storage_name">{{ repository.storage_name }}</div>
        <div class="col">{{ repository.repository_type }}</div>
        <div class="col">{{ repository.active }}</div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import router from '@/router'
import type { RepositoryWithStorageName } from '@/types/repository'
import { computed, ref, type PropType } from 'vue'
const searchValue = ref<string>('')

const props = defineProps({
  repositories: Array as PropType<RepositoryWithStorageName[]>
})
const sortBy = ref<string>('id')

function sortList(a: RepositoryWithStorageName, b: RepositoryWithStorageName) {
  switch (sortBy.value) {
    case 'id':
      return a.name.localeCompare(b.name)
    case 'name':
      return a.name.localeCompare(b.name)

    default:
      return 0
  }
}
const filteredTable = computed(() => {
  if (props.repositories == undefined) {
    return []
  }
  const users = props.repositories.map((user) => user)
  return users.sort(sortList)
})
</script>
<style scoped lang="scss">
@import '@/assets/styles/theme';
#headerBar {
  display: flex;
  justify-content: space-between;
  padding: 1rem;
  background-color: $primary-30;
  input {
    width: 25%;
  }
}
@media screen and (max-width: 1200px) {
  #headerBar {
    input {
      width: 50%;
    }
  }
}
@media screen and (max-width: 800px) {
  #headerBar {
    display: flex;
    flex-direction: column;
    input {
      width: 100%;
    }
  }
}
#storages {
  background-color: $primary-50;
}

#header {
  .col {
    font-weight: bold;
    &:hover {
      cursor: pointer;
      color: $accent;
      transition: all 0.3s ease;
    }
  }
}
.row {
  display: grid;
  grid-template-columns: 1fr 0.5fr 0.5fr 0.5fr 0.5fr;
  grid-template-rows: auto;
  .col {
    padding: 1rem;
    border-bottom: 1px solid $primary-30;
  }
}
.item {
  cursor: pointer;
  &:hover {
    background-color: $primary-30;
    transition: all 0.3s ease;
    .col {
      color: $accent;
      transition: all 0.3s ease;
    }
  }
}
</style>
