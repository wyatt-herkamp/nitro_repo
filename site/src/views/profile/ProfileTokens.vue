<template>
  <main class="container">
    <div class="page-header">
      <div class="page-header-text">
        <h1>API tokens</h1>
        <p>Tokens authenticate `mvn`, `npm` and CI against this instance.</p>
      </div>
      <div class="page-header-actions">
        <NButton
          variant="primary"
          icon="plus"
          :to="{ name: 'profileTokenCreate' }">
          New token
        </NButton>
        <NButton
          v-if="tokens.length > 0"
          variant="danger"
          @click="confirmingRevokeAll = true">
          Revoke all
        </NButton>
      </div>
    </div>

    <NCard flush>
      <NEmptyState
        v-if="!loading && tokens.length === 0"
        title="No API tokens"
        description="Create one to publish or pull from a private repository without your password."
        icon="key">
        <NButton
          variant="primary"
          :to="{ name: 'profileTokenCreate' }"
          >Create a token</NButton
        >
      </NEmptyState>

      <ul
        v-else
        class="tokenList">
        <li
          v-for="entry in tokens"
          :key="entry.token.id"
          class="tokenItem">
          <button
            type="button"
            class="tokenSummary"
            :aria-expanded="expanded === entry.token.id"
            @click="toggle(entry.token.id)">
            <span class="tokenName">
              {{ entry.token.name || "Unnamed token" }}
              <NBadge
                v-if="isExpired(entry.token)"
                variant="danger"
                >Expired</NBadge
              >
              <NBadge
                v-else-if="!entry.token.active"
                variant="neutral"
                >Inactive</NBadge
              >
            </span>

            <span class="tokenFacts">
              <span class="fact">
                <span class="factLabel">Source</span>
                <span>{{ entry.token.source }}</span>
              </span>
              <span class="fact">
                <span class="factLabel">Created</span>
                <span>{{ formatDate(entry.token.created_at) }}</span>
              </span>
              <span class="fact">
                <span class="factLabel">Expires</span>
                <span>{{
                  entry.token.expires_at ? formatDate(entry.token.expires_at) : "Never"
                }}</span>
              </span>
              <span class="fact">
                <span class="factLabel">Last used</span>
                <span>{{ formatRelative(entry.token.last_used_at) }}</span>
              </span>
            </span>
          </button>

          <div
            v-if="expanded === entry.token.id"
            class="tokenDetail">
            <p
              v-if="entry.token.description"
              class="description">
              {{ entry.token.description }}
            </p>

            <div class="detailSection">
              <span class="label">Scopes</span>
              <div
                v-if="entry.scopes.length > 0"
                class="badges">
                <NBadge
                  v-for="scope in entry.scopes"
                  :key="scope.id"
                  variant="accent">
                  {{ scopeLabel(scope.scope) }}
                </NBadge>
              </div>
              <p
                v-else
                class="muted">
                No account-wide scopes.
              </p>
            </div>

            <div class="detailSection">
              <span class="label">Repository access</span>
              <div
                v-if="entry.repository_scopes.length > 0"
                class="repositoryScopes">
                <div
                  v-for="scope in entry.repository_scopes"
                  :key="scope.id"
                  class="repositoryScope">
                  <span class="mono">{{ repositoryName(String(scope.repository_id)) }}</span>
                  <div class="badges">
                    <NBadge
                      v-for="action in scope.actions"
                      :key="action"
                      >{{ action }}</NBadge
                    >
                  </div>
                </div>
              </div>
              <p
                v-else
                class="muted">
                No repository-specific access.
              </p>
            </div>

            <NButton
              variant="danger"
              size="sm"
              icon="trash"
              @click="askDelete(entry)">
              Delete this token
            </NButton>
          </div>
        </li>
      </ul>
    </NCard>

    <NConfirmDialog
      v-model="confirmingDelete"
      title="Delete this token?"
      :message="`Anything still authenticating with ${pendingDelete?.token.name || 'this token'} stops working immediately.`"
      confirm-label="Delete token"
      destructive
      :loading="deleting"
      @confirm="deleteToken" />

    <NConfirmDialog
      v-model="confirmingRevokeAll"
      title="Revoke every token?"
      message="Every API token on your account stops working immediately — CI jobs, a local mvn or npm, anything holding one."
      confirm-label="Revoke all tokens"
      destructive
      confirm-text="revoke"
      :loading="revoking"
      @confirm="revokeAll" />
  </main>
</template>

<script setup lang="ts">
/**
 * The API returns scopes, expiry and last-used for every token, and none of it used to be shown —
 * so there was no way to tell what a token could do or whether anything still used it. Deleting one
 * also fired straight from the click handler with no confirmation.
 */
import { ref } from "vue";
import { notify } from "@kyvg/vue3-notification";
import NCard from "@/components/core/ui/NCard.vue";
import NButton from "@/components/core/ui/NButton.vue";
import NBadge from "@/components/core/ui/NBadge.vue";
import NEmptyState from "@/components/core/ui/NEmptyState.vue";
import NConfirmDialog from "@/components/core/ui/NConfirmDialog.vue";
import http from "@/http";
import { formatDate, formatRelative } from "@/utils/format";
import { useRepositoryStore } from "@/stores/repositories";
import { sessionStore } from "@/stores/session";
import type { ScopeDescription } from "@/types/base";
import type { RawAuthTokenFullResponse, RawAuthTokenResponse } from "@/types/user/token";

const session = sessionStore();
const repositoryStore = useRepositoryStore();

const tokens = ref<Array<RawAuthTokenFullResponse>>([]);
const scopeDescriptions = ref<Array<ScopeDescription>>([]);
const expanded = ref<number | undefined>(undefined);
const loading = ref(true);

const confirmingDelete = ref(false);
const confirmingRevokeAll = ref(false);
const pendingDelete = ref<RawAuthTokenFullResponse | undefined>(undefined);
const deleting = ref(false);
const revoking = ref(false);

function toggle(id: number) {
  expanded.value = expanded.value === id ? undefined : id;
}

function isExpired(token: RawAuthTokenResponse): boolean {
  return token.expires_at !== undefined && new Date(token.expires_at).getTime() < Date.now();
}

function scopeLabel(key: string): string {
  return scopeDescriptions.value.find((scope) => scope.key === key)?.name ?? key;
}

// A repository scope stores an id; showing the id alone gives no idea which repository it grants.
function repositoryName(id: string): string {
  return repositoryStore.getRepositoryFromCache(id)?.name ?? id;
}

function askDelete(entry: RawAuthTokenFullResponse) {
  pendingDelete.value = entry;
  confirmingDelete.value = true;
}

async function deleteToken() {
  const target = pendingDelete.value;
  if (!target) return;

  deleting.value = true;
  try {
    await http.delete(`/api/user/token/delete/${target.token.id}`);
    notify({ type: "success", title: "Token deleted" });
    await load();
  } catch {
    notify({ type: "error", title: "Could not delete the token" });
  } finally {
    deleting.value = false;
    confirmingDelete.value = false;
    pendingDelete.value = undefined;
  }
}

async function revokeAll() {
  revoking.value = true;
  try {
    const response = await http.delete<{ revoked: number }>("/api/user/token/revoke-all");
    notify({
      type: "success",
      title: "Tokens revoked",
      text: `${response.data.revoked} token(s) revoked.`,
    });
    await load();
  } catch {
    notify({ type: "error", title: "Could not revoke tokens" });
  } finally {
    revoking.value = false;
    confirmingRevokeAll.value = false;
  }
}

async function load() {
  if (session.user === undefined) return;
  try {
    const [tokenResponse, scopeResponse] = await Promise.all([
      http.get<Array<RawAuthTokenFullResponse>>("/api/user/token/list"),
      http.get<Array<ScopeDescription>>("/api/info/scopes"),
    ]);
    tokens.value = tokenResponse.data;
    scopeDescriptions.value = scopeResponse.data;
    // Populates the cache the repository-scope labels read from.
    await repositoryStore.getRepositories(false);
  } catch {
    notify({ type: "error", title: "Could not load your tokens" });
  } finally {
    loading.value = false;
  }
}
load();
</script>

<style scoped lang="scss">
.tokenList {
  list-style: none;
  margin: 0;
  padding: 0;
}

.tokenItem {
  border-bottom: 1px solid var(--border);

  &:last-child {
    border-bottom: none;
  }
}

.tokenSummary {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-4);
  width: 100%;
  padding: var(--space-3) var(--space-4);
  font: inherit;
  color: inherit;
  text-align: left;
  background: none;
  border: none;
  cursor: pointer;

  &:hover {
    background-color: var(--surface-hover);
  }
}

.tokenName {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  font-weight: var(--weight-medium);
}

.tokenFacts {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-5);
}

.fact {
  display: flex;
  flex-direction: column;
  font-size: var(--text-sm);
}

.factLabel {
  font-size: var(--text-2xs);
  letter-spacing: var(--tracking-label);
  text-transform: uppercase;
  color: var(--text-subtle);
}

.tokenDetail {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: var(--space-4);
  padding: 0 var(--space-4) var(--space-4);
}

.description {
  font-size: var(--text-sm);
  color: var(--text-muted);
}

.detailSection {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.badges {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-1);
}

.repositoryScopes {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.repositoryScope {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  font-size: var(--text-sm);
}
</style>
