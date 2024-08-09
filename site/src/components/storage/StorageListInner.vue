<template>
  <div id="storagesBox">
    <div id="headerBar">
      <h2>Storages</h2>
      <input
        type="text"
        id="nameSearch"
        v-model="searchValue"
        autofocus
        placeholder="Search by Name, Username, or Primary Email Address"
      />
    </div>
    <div id="storages" class="betterScroll">
      <div class="row" id="header">
        <div
          :class="['col', { sorted: sortBy === 'id' }]"
          @click="sortBy = 'id'"
          title="Sort by ID"
        >
          ID #
        </div>
        <div
          :class="['col', { sorted: sortBy === 'name' }]"
          @click="sortBy = 'name'"
          title="Sort by Name"
        >
          Name
        </div>
        <div
          :class="['col', { sorted: sortBy === 'storage-type' }]"
          @click="sortBy = 'storage-type'"
          title="Sort by Storage Type"
        >
          Storage Type
        </div>
        <div :class="['col']">Active</div>
      </div>
      <div
        class="row item"
        v-for="storage in filteredTable"
        :key="storage.id"
        @click="router.push({ name: 'ViewStorage', params: { id: storage.id } })"
      >
        <div class="col">{{ storage.id }}</div>
        <div class="col" :title="storage.name">{{ storage.name }}</div>
        <div class="col" :title="storage.storage_type">{{ storage.storage_type }}</div>
        <div class="col">{{ storage.active }}</div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import router from '@/router'
import type { User } from '@/types/base'
import type { StorageItem } from '@/types/storage'
import { notify } from '@kyvg/vue3-notification'
import { computed, ref, type PropType } from 'vue'
const searchValue = ref<string>('')

const props = defineProps({
  storages: Array as PropType<StorageItem[]>
})
const sortBy = ref<string>('id')

function sortList(a: StorageItem, b: StorageItem) {
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
  if (props.storages == undefined) {
    return []
  }
  const users = props.storages.map((user) => user)
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
  grid-template-columns: 1fr 0.5fr 0.5fr 0.5fr;
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
