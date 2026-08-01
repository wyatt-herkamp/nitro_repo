<template>
  <NCard
    title="Storages"
    flush>
    <template
      v-if="$slots.actions"
      #actions>
      <slot name="actions" />
    </template>

    <NDataTable
      :columns="columns"
      :rows="storages"
      row-key="id"
      clickable
      :loading="loading"
      :error="error"
      search-placeholder="Search by name or type"
      empty-title="No storages"
      empty-description="A storage is where a repository's files physically live."
      empty-icon="database"
      @row-click="open">
      <template #cell-storage_type="{ value }">
        <NBadge variant="accent">{{ value }}</NBadge>
      </template>

      <template #cell-active="{ value }">
        <NBadge
          :variant="value ? 'success' : 'neutral'"
          dot>
          {{ value ? "Active" : "Disabled" }}
        </NBadge>
      </template>

      <template #cell-created_at="{ value }">
        {{ formatDate(value) }}
      </template>
    </NDataTable>
  </NCard>
</template>

<script setup lang="ts">
import { useRouter } from "vue-router";
import NCard from "@/components/core/ui/NCard.vue";
import NDataTable, { type Column } from "@/components/core/ui/NDataTable.vue";
import NBadge from "@/components/core/ui/NBadge.vue";
import { formatDate } from "@/utils/format";
import type { StorageItem } from "./storageTypes";

withDefaults(
  defineProps<{
    storages: Array<StorageItem>;
    loading?: boolean;
    error?: string;
  }>(),
  { loading: false, error: undefined },
);

const router = useRouter();

const columns: Array<Column<StorageItem>> = [
  { key: "name", label: "Name", sortable: true },
  { key: "storage_type", label: "Type", sortable: true },
  { key: "active", label: "Status", sortable: true, searchable: false },
  { key: "created_at", label: "Created", sortable: true, searchable: false, mono: true },
];

function open(storage: StorageItem) {
  router.push({ name: "ViewStorage", params: { id: storage.id } });
}
</script>
