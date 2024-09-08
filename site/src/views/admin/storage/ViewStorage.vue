<template>
  <main>
    <form v-if="storage" id="storage">
      <TwoByFormBox>
        <TextInput v-model="storage.name" required disabled> Name </TextInput>
        <TextInput v-model="storage.storage_type" disabled> Storage Type </TextInput>
      </TwoByFormBox>
      <component :is="storageComponent" v-model="storage.config.settings"></component>
    </form>
  </main>
</template>
<script setup lang="ts">
import DropDown from '@/components/form/dropdown/DropDown.vue'
import TextInput from '@/components/form/text/TextInput.vue'
import TwoByFormBox from '@/components/form/TwoByFormBox.vue'
import { storageTypes, type StorageItem } from '@/components/nr/storage/storageTypes'
import http from '@/http'
import router from '@/router'
import { computed, ref } from 'vue'

const storageId = router.currentRoute.value.params.id as string

const storage = ref<StorageItem | undefined>(undefined)
const storageComponent = computed(() => {
  if (!storage.value) {
    return undefined
  }
  console.log('Value ' + JSON.stringify(storage.value.config.settings))
  return storageTypes.find((type) => type.value === storage.value?.storage_type)?.updateComponent
})
async function getStorage() {
  await http.get(`/api/storage/${storageId}`).then((response) => {
    storage.value = response.data
  })
}
getStorage()
</script>
<style scoped lang="scss">
#storage {
  padding: 1rem;
}
</style>
