import LocalStorageConfig from '@/components/storage/local/LocalStorageConfig.vue'
import UpdateLocalStorageConfig from '@/components/storage/local/UpdateLocalStorageConfig.vue'

interface LocalConfig {
  path: string
}
type StorageTypeConfig = {
  type: 'Local'
  settings: LocalConfig
}
export interface StorageItem {
  id: string
  name: string
  storage_type: string
  config: StorageTypeConfig
  active: boolean
  created_at: Date
}

export const storageTypes = [
  {
    label: 'Local',
    value: 'Local',
    title: 'Local Storage Configuration',
    component: LocalStorageConfig,
    updateComponent: UpdateLocalStorageConfig
  }
]
