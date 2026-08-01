<template>
  <NCard
    :title="title"
    flush>
    <template
      v-if="$slots.actions"
      #actions>
      <slot name="actions" />
    </template>

    <NDataTable
      :columns="columns"
      :rows="repositories"
      row-key="id"
      clickable
      :loading="loading"
      :error="error"
      search-placeholder="Search by name, storage, or type"
      empty-title="No repositories"
      empty-description="Repositories hold the artifacts clients push and pull."
      empty-icon="box-open"
      @row-click="open">
      <template #cell-name="{ row }">
        <span class="repoName">{{ row.name }}</span>
      </template>

      <template #cell-repository_type="{ value }">
        <NBadge variant="accent">{{ value }}</NBadge>
      </template>

      <template #cell-visibility="{ value }">
        <NBadge :variant="visibilityVariant(String(value))">{{ value }}</NBadge>
      </template>

      <template #cell-active="{ value }">
        <NBadge
          :variant="value ? 'success' : 'neutral'"
          dot>
          {{ value ? "Active" : "Disabled" }}
        </NBadge>
      </template>
    </NDataTable>
  </NCard>
</template>

<script setup lang="ts">
/**
 * The repository list.
 *
 * `PublicRepositoryList` and `RepositoryListInner` were two near-identical copies of the same
 * hand-built CSS grid — both with a search box wired to a ref no filter ever read, both carrying the
 * user list's "Search by Name, Username, or Primary Email Address" placeholder, and the admin one
 * sorting by name under `case "id"`. This is the one of them.
 */
import { useRouter } from "vue-router";
import NCard from "@/components/core/ui/NCard.vue";
import NDataTable, { type Column } from "@/components/core/ui/NDataTable.vue";
import NBadge from "@/components/core/ui/NBadge.vue";
import type { RepositoryWithStorageName } from "@/types/repository";

const props = withDefaults(
  defineProps<{
    repositories: Array<RepositoryWithStorageName>;
    loading?: boolean;
    error?: string;
    title?: string;
    /** Admin rows go to the settings page; public ones go to browse. */
    admin?: boolean;
  }>(),
  { loading: false, error: undefined, title: "Repositories", admin: false },
);

const router = useRouter();

const columns: Array<Column<RepositoryWithStorageName>> = [
  { key: "name", label: "Name", sortable: true },
  { key: "storage_name", label: "Storage", sortable: true },
  { key: "repository_type", label: "Type", sortable: true },
  { key: "visibility", label: "Visibility", sortable: true },
  { key: "active", label: "Status", sortable: true, searchable: false },
];

function visibilityVariant(visibility: string) {
  if (visibility === "Public") return "success";
  if (visibility === "Hidden") return "warning";
  return "neutral";
}

function open(repository: RepositoryWithStorageName) {
  if (props.admin) {
    router.push({ name: "AdminViewRepository", params: { id: repository.id } });
  } else {
    router.push({ name: "Browse", params: { id: repository.id } });
  }
}
</script>

<style scoped lang="scss">
.repoName {
  font-weight: var(--weight-medium);
}
</style>
