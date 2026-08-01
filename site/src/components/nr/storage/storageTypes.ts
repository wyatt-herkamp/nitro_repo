import type { Component } from "vue";
import LocalStorageConfig from "@/components/nr/storage/local/LocalStorageConfig.vue";
import UpdateLocalStorageConfig from "@/components/nr/storage/local/UpdateLocalStorageConfig.vue";
import S3StorageConfig from "@/components/nr/storage/s3/S3StorageConfig.vue";
import FileSystemV2Config from "@/components/nr/storage/fsv2/FileSystemV2Config.vue";

export interface StorageType {
  label: string;
  value: string;
  title: string;
  description: string;
  component: Component;
  /** The form shown when editing an existing storage. */
  updateComponent: Component;
  /** A sensible starting config, so a new storage form is never empty. */
  defaultConfig: () => Record<string, unknown>;
}

/**
 * The storage backends offered in the UI.
 *
 * This listed only `Local`, so the S3 and FileSystemV2 backends built in Phase 1 could not be
 * created through the interface at all — `POST /api/storage/new/{type}` accepted them, but nothing
 * in the frontend would ever call it with those names. The values must match the
 * `StorageTypeConfig` serde tags in `crates/storage/src/config.rs`.
 */
export const storageTypes: Array<StorageType> = [
  {
    label: "Local",
    value: "Local",
    title: "Local storage",
    description: "Files on the server's filesystem, with metadata in .nr-meta sidecar files.",
    component: LocalStorageConfig,
    updateComponent: UpdateLocalStorageConfig,
    defaultConfig: () => ({ path: "" }),
  },
  {
    label: "FileSystem V2",
    value: "FileSystemV2",
    title: "FileSystem V2",
    description:
      "Files on the server's filesystem, one self-contained object per file — no sidecars, " +
      "optional compression, and support for HTTP range requests.",
    component: FileSystemV2Config,
    updateComponent: FileSystemV2Config,
    defaultConfig: () => ({ path: "", compression: "None", sync: false }),
  },
  {
    label: "S3",
    value: "S3",
    title: "S3",
    description: "An S3 bucket, or any S3-compatible service such as MinIO, Ceph or R2.",
    component: S3StorageConfig,
    updateComponent: S3StorageConfig,
    defaultConfig: () => ({
      bucket_name: "",
      region: "UsEast1",
      credentials: {},
      path_style: false,
    }),
  },
];

export function getStorageType(value: string): StorageType | undefined {
  return storageTypes.find((type) => type.value === value);
}

export interface LocalConfig {
  path: string;
}

export interface FileSystemV2ConfigType {
  path: string;
  compression: "None" | "Zstd" | "Gzip";
  sync: boolean;
}

export interface S3ConfigType {
  bucket_name: string;
  region?: string;
  /** Flattened into `S3Config` on the server, so these sit at the top level rather than nested. */
  custom_region?: string;
  endpoint?: string;
  credentials: { access_key?: string; secret_key?: string };
  path_style: boolean;
}

export type StorageTypeConfig =
  | { type: "Local"; settings: LocalConfig }
  | { type: "FileSystemV2"; settings: FileSystemV2ConfigType }
  | { type: "S3"; settings: S3ConfigType };

export interface StorageItem {
  id: string;
  name: string;
  storage_type: string;
  config: StorageTypeConfig;
  active: boolean;
  created_at: Date;
}
