export interface Storage {
  id: string;
  created: number;
  handler_config: unknown;
}

export interface StorageList {
  storages: Array<Storage>;
}
