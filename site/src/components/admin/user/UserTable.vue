<template>
  <NCard
    title="Users"
    flush>
    <template
      v-if="$slots.actions"
      #actions>
      <slot name="actions" />
    </template>

    <NDataTable
      :columns="columns"
      :rows="users"
      row-key="id"
      clickable
      :loading="loading"
      :error="error"
      search-placeholder="Search by name, username, or email"
      empty-title="No users"
      empty-icon="users"
      @row-click="open">
      <template #cell-name="{ row }">
        <div class="userCell">
          <span class="userName">{{ row.name }}</span>
          <span class="userHandle">{{ row.username }}</span>
        </div>
      </template>

      <template #cell-roles="{ row }">
        <div class="roles">
          <NBadge
            v-if="row.admin"
            variant="danger"
            >Admin</NBadge
          >
          <NBadge
            v-if="row.user_manager"
            variant="info"
            >User manager</NBadge
          >
          <NBadge
            v-if="row.system_manager"
            variant="info"
            >System manager</NBadge
          >
          <span
            v-if="!row.admin && !row.user_manager && !row.system_manager"
            class="muted"
            >—</span
          >
        </div>
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
import type { UserResponseType } from "@/types/base";

withDefaults(
  defineProps<{
    users: Array<UserResponseType>;
    loading?: boolean;
    error?: string;
  }>(),
  { loading: false, error: undefined },
);

const router = useRouter();

const columns: Array<Column<UserResponseType>> = [
  { key: "name", label: "User", sortable: true },
  { key: "username", label: "Username", sortable: true, mono: true },
  { key: "email", label: "Email", sortable: true },
  { key: "roles", label: "Roles", searchable: false, value: () => "" },
  { key: "created_at", label: "Joined", sortable: true, searchable: false, mono: true },
];

function open(user: UserResponseType) {
  router.push(`/admin/user/${user.id}`);
}
</script>

<style scoped lang="scss">
.userCell {
  display: flex;
  flex-direction: column;
}

.userName {
  font-weight: var(--weight-medium);
}

.userHandle {
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  color: var(--text-subtle);
}

.roles {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-1);
}
</style>
