import LocalStorageConfig from "@/components/nr/storage/local/LocalStorageConfig.vue";
import UpdateLocalStorageConfig from "@/components/nr/storage/local/UpdateLocalStorageConfig.vue";

interface StorageType {
  label: string;
  value: string;
  title: string;
  description: string;
  component: any;
  updateComponent: any;
}
export const storageTypes: Array<StorageType> = [
  {
    label: "Local",
    value: "Local",
    title: "Local Storage Configuration",
    description: "Local storage configuration allows you to store data on your local machine.",
    component: LocalStorageConfig,
    updateComponent: UpdateLocalStorageConfig,
  },
];

export function getStorageType(value: string): StorageType | undefined {
  return storageTypes.find((type) => type.value === value);
}

export interface LocalConfig {
  path: string;
}
export type StorageTypeConfig = {
  type: "Local";
  settings: LocalConfig;
};
export interface StorageItem {
  id: string;
  name: string;
  storage_type: string;
  config: StorageTypeConfig;
  active: boolean;
  created_at: Date;
}
