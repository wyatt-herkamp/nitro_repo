export interface Storage {
  id: string;
  created: number;
}

export interface StorageList {
  storages: Array<Storage>;
}
