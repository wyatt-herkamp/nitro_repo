export interface Storage {
  id: string;
  created: Date;
  handler_config: StorageConfig;
}

export type StorageConfig = {
  storage_type: "LocalStorage";
  settings: {
    location: string;
  };
};

export interface StorageList {
  storages: Array<Storage>;
}
