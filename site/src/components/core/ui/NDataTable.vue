<template>
  <div class="nDataTable">
    <div
      v-if="searchable || $slots.actions"
      class="tableToolbar">
      <div
        v-if="searchable"
        class="searchWrapper">
        <font-awesome-icon
          icon="magnifying-glass"
          class="searchIcon" />
        <input
          v-model="search"
          type="search"
          class="searchInput"
          :placeholder="searchPlaceholder"
          :aria-label="searchPlaceholder" />
      </div>
      <div
        v-if="$slots.actions"
        class="toolbarActions">
        <slot name="actions" />
      </div>
    </div>

    <div class="tableScroll">
      <table>
        <thead>
          <tr>
            <th
              v-for="column in columns"
              :key="column.key"
              :style="{ width: column.width, textAlign: column.align ?? 'left' }"
              :aria-sort="ariaSort(column)">
              <button
                v-if="column.sortable"
                type="button"
                class="sortButton"
                @click="toggleSort(column.key)">
                {{ column.label }}
                <font-awesome-icon
                  v-if="sortKey === column.key"
                  :icon="sortDirection === 'asc' ? 'arrow-up' : 'arrow-down'"
                  class="sortIcon" />
              </button>
              <span v-else>{{ column.label }}</span>
            </th>
          </tr>
        </thead>

        <tbody v-if="loading">
          <tr
            v-for="index in skeletonRows"
            :key="index"
            class="skeletonRow">
            <td
              v-for="column in columns"
              :key="column.key">
              <NSkeleton height="0.875rem" />
            </td>
          </tr>
        </tbody>

        <tbody v-else>
          <tr
            v-for="(row, index) in visibleRows"
            :key="keyFor(row, index)"
            :class="{ clickable: clickable }"
            :tabindex="clickable ? 0 : undefined"
            @click="clickable && emit('rowClick', row)"
            @keydown.enter="clickable && emit('rowClick', row)">
            <td
              v-for="column in columns"
              :key="column.key"
              :style="{ textAlign: column.align ?? 'left' }"
              :class="{ mono: column.mono }">
              <slot
                :name="`cell-${column.key}`"
                :row="row"
                :value="valueOf(row, column)">
                {{ display(valueOf(row, column)) }}
              </slot>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <div
      v-if="!loading && error"
      class="tableMessage isError">
      <font-awesome-icon icon="circle-xmark" />
      <span>{{ error }}</span>
    </div>

    <NEmptyState
      v-else-if="!loading && rows.length === 0"
      :title="emptyTitle"
      :description="emptyDescription"
      :icon="emptyIcon" />

    <!-- A search that matches nothing is a different situation from having no data at all: the fix
         is to change the search, not to go and create something. -->
    <div
      v-else-if="!loading && visibleRows.length === 0"
      class="tableMessage">
      <span
        >No results for <strong>{{ search }}</strong></span
      >
      <NButton
        size="sm"
        variant="ghost"
        @click="search = ''"
        >Clear search</NButton
      >
    </div>
  </div>
</template>

<script setup lang="ts" generic="T extends object">
/**
 * A sortable, searchable table.
 *
 * This replaces four hand-copied CSS-grid tables. Each had its own `searchValue` ref bound to a
 * search box and never referenced by the filter, so none of the four search boxes did anything; the
 * repository one also carried the user list's placeholder ("Search by Name, Username, or Primary
 * Email Address") and sorted by name under `case "id"`.
 *
 * A real `<table>` rather than grid-of-divs, so that a screen reader announces rows and columns and
 * so that column headers are actual headers.
 */
import { computed, ref } from "vue";
import NSkeleton from "./NSkeleton.vue";
import NEmptyState from "./NEmptyState.vue";
import NButton from "./NButton.vue";

export interface Column<Row> {
  key: string;
  label: string;
  sortable?: boolean;
  width?: string;
  align?: "left" | "right" | "center";
  /** Renders the cell in the mono face — for ids, versions, coordinates, sizes. */
  mono?: boolean;
  /** Pulls the value out of the row. Defaults to `row[key]`. */
  value?: (row: Row) => unknown;
  /** Excludes the column from the search. Defaults to searchable. */
  searchable?: boolean;
}

const props = withDefaults(
  defineProps<{
    columns: Array<Column<T>>;
    rows: Array<T>;
    rowKey?: keyof T | ((row: T) => string | number);
    loading?: boolean;
    error?: string;
    searchable?: boolean;
    searchPlaceholder?: string;
    clickable?: boolean;
    emptyTitle?: string;
    emptyDescription?: string;
    emptyIcon?: string | Array<string>;
    skeletonRows?: number;
  }>(),
  {
    rowKey: undefined,
    loading: false,
    error: undefined,
    searchable: true,
    searchPlaceholder: "Search",
    clickable: false,
    emptyTitle: "Nothing here yet",
    emptyDescription: undefined,
    emptyIcon: undefined,
    skeletonRows: 5,
  },
);

const emit = defineEmits<{ (e: "rowClick", row: T): void }>();

const search = ref("");
const sortKey = ref<string | undefined>(undefined);
const sortDirection = ref<"asc" | "desc">("asc");

function valueOf(row: T, column: Column<T>): unknown {
  // `T extends object` rather than `Record<string, unknown>`: an interface has no index
  // signature, so the stricter constraint rejected every real row type in the app.
  return column.value ? column.value(row) : (row as Record<string, unknown>)[column.key];
}

function display(value: unknown): string {
  if (value === undefined || value === null) return "—";
  if (typeof value === "boolean") return value ? "Yes" : "No";
  return String(value);
}

function keyFor(row: T, index: number): string | number {
  if (typeof props.rowKey === "function") return props.rowKey(row);
  if (props.rowKey) return row[props.rowKey] as string | number;
  return index;
}

function toggleSort(key: string) {
  if (sortKey.value === key) {
    sortDirection.value = sortDirection.value === "asc" ? "desc" : "asc";
  } else {
    sortKey.value = key;
    sortDirection.value = "asc";
  }
}

function ariaSort(column: Column<T>) {
  if (!column.sortable) return undefined;
  if (sortKey.value !== column.key) return "none";
  return sortDirection.value === "asc" ? "ascending" : "descending";
}

const visibleRows = computed(() => {
  let result = [...props.rows];

  const term = search.value.trim().toLowerCase();
  if (term) {
    const searchColumns = props.columns.filter((column) => column.searchable !== false);
    result = result.filter((row) =>
      searchColumns.some((column) => {
        const value = valueOf(row, column);
        return value !== undefined && value !== null && String(value).toLowerCase().includes(term);
      }),
    );
  }

  const key = sortKey.value;
  if (key) {
    const column = props.columns.find((candidate) => candidate.key === key);
    if (column) {
      const direction = sortDirection.value === "asc" ? 1 : -1;
      result.sort((a, b) => compare(valueOf(a, column), valueOf(b, column)) * direction);
    }
  }

  return result;
});

/** Numbers compare numerically and everything else compares as a locale-aware string. */
function compare(a: unknown, b: unknown): number {
  if (a === b) return 0;
  if (a === undefined || a === null) return 1;
  if (b === undefined || b === null) return -1;
  if (typeof a === "number" && typeof b === "number") return a - b;
  if (typeof a === "boolean" && typeof b === "boolean") return a === b ? 0 : a ? -1 : 1;
  return String(a).localeCompare(String(b), undefined, { numeric: true, sensitivity: "base" });
}
</script>

<style scoped lang="scss">
.nDataTable {
  display: flex;
  flex-direction: column;
}

.tableToolbar {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
  padding: var(--space-3) var(--space-4);
  border-bottom: 1px solid var(--border);
}

.searchWrapper {
  position: relative;
  flex: 1;
  min-width: 12rem;
  max-width: 22rem;
}

.searchIcon {
  position: absolute;
  left: 0.625rem;
  top: 50%;
  transform: translateY(-50%);
  color: var(--text-subtle);
  font-size: var(--text-xs);
  pointer-events: none;
}

.searchInput {
  padding-left: 2rem;
  font-size: var(--text-sm);
}

.toolbarActions {
  display: flex;
  gap: var(--space-2);
}

// A table with many columns should scroll inside its own box rather than pushing the page sideways.
.tableScroll {
  overflow-x: auto;
}

table {
  width: 100%;
  border-collapse: collapse;
  font-size: var(--text-sm);
}

th {
  padding: var(--space-3) var(--space-4);
  font-size: var(--text-xs);
  font-weight: var(--weight-semibold);
  letter-spacing: var(--tracking-label);
  text-transform: uppercase;
  color: var(--text-subtle);
  white-space: nowrap;
  border-bottom: 1px solid var(--border);
  background-color: var(--bg-sunken);
}

td {
  padding: var(--space-3) var(--space-4);
  border-bottom: 1px solid var(--border);
  color: var(--text);
}

tbody tr:last-child td {
  border-bottom: none;
}

.mono {
  font-family: var(--font-mono);
  font-variant-numeric: tabular-nums;
  font-size: var(--text-xs);
}

.sortButton {
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
  padding: 0;
  font: inherit;
  letter-spacing: inherit;
  text-transform: inherit;
  color: inherit;
  background: none;
  border: none;
  cursor: pointer;

  &:hover {
    color: var(--accent);
  }
}

.sortIcon {
  font-size: 0.625rem;
  color: var(--accent);
}

.clickable {
  cursor: pointer;
  transition: background-color var(--duration-fast) var(--ease-out);

  &:hover,
  &:focus-visible {
    background-color: var(--surface-hover);
  }
}

.skeletonRow td {
  // Keeps the placeholder row the same height as a real one, so the table does not jump when it
  // resolves.
  padding-block: calc(var(--space-3) + 0.0625rem);
}

.tableMessage {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-3);
  padding: var(--space-8) var(--space-4);
  font-size: var(--text-sm);
  color: var(--text-muted);

  &.isError {
    color: var(--danger);
  }
}
</style>
