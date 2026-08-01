<template>
  <main>
    <div class="header">
      <h1>API Tokens</h1>
      <SubmitButton
        v-if="authTokens.length > 0"
        class="dangerButton"
        @click="revokeAll"
        >Revoke all tokens</SubmitButton
      >
    </div>
    <p v-if="authTokens.length === 0">You have no API tokens.</p>
    <ul class="tokenList">
      <li
        v-for="token in authTokens"
        :key="token.token.id"
        class="tokenElement"
        :data-token-active="token.token.active"
        :data-expanded="expandedToken == token.token.id"
        @click="tokenClicked(token.token.id)">
        <div class="tokenElementLine">
          <KeyAndValue
            label="Name"
            :value="token.token.name || 'No name'" />
          <KeyAndValue
            label="Source"
            :value="token.token.source" />
          <KeyAndValue
            label="Created On"
            :value="formatDate(token.token.created_at)" />
          <KeyAndValue
            label="Expires"
            :value="expiryLabel(token.token)" />
          <KeyAndValue
            label="Last Used"
            :value="token.token.last_used_at ? formatDate(token.token.last_used_at) : 'Never'" />
        </div>
        <div
          v-if="expandedToken == token.token.id"
          class="tokenDetail">
          <div v-if="token.token.description">
            <h3>Description</h3>
            <p>{{ token.token.description }}</p>
          </div>
          <div>
            <h3>Scopes</h3>
            <ul
              v-if="token.scopes.length > 0"
              class="scopeList">
              <li
                v-for="scope in token.scopes"
                :key="scope.id">
                {{ scopeLabel(scope.scope) }}
              </li>
            </ul>
            <p v-else>No account-wide scopes.</p>
          </div>
          <div>
            <h3>Repository Access</h3>
            <ul
              v-if="token.repository_scopes.length > 0"
              class="scopeList">
              <li
                v-for="repositoryScope in token.repository_scopes"
                :key="repositoryScope.id">
                {{ repositoryScope.repository_id }} — {{ repositoryScope.actions.join(", ") }}
              </li>
            </ul>
            <p v-else>No repository-specific access.</p>
          </div>
          <SubmitButton @click.stop="deleteToken(token.token.id)">Delete</SubmitButton>
        </div>
      </li>
    </ul>
  </main>
</template>
<script setup lang="ts">
import KeyAndValue from "@/components/form/KeyAndValue.vue";
import SubmitButton from "@/components/form/SubmitButton.vue";
import http from "@/http";
import { sessionStore } from "@/stores/session";
import type { ScopeDescription } from "@/types/base";
import { type RawAuthTokenFullResponse, type RawAuthTokenResponse } from "@/types/user/token";
import { notify } from "@kyvg/vue3-notification";
import { ref } from "vue";

const session = sessionStore();
const user = session.user;
const authTokens = ref<Array<RawAuthTokenFullResponse>>([]);
const scopeDescriptions = ref<Array<ScopeDescription>>([]);
const expandedToken = ref<number | undefined>(undefined);

function tokenClicked(tokenId: number) {
  expandedToken.value = expandedToken.value == tokenId ? undefined : tokenId;
}
function formatDate(value: string): string {
  return new Date(value).toLocaleString();
}
// The API returns scopes, expiry and last-used for every token; none of it was displayed, which
// left no way to tell what a token could do or whether it was still in use.
function expiryLabel(token: RawAuthTokenResponse): string {
  if (!token.expires_at) {
    return "Never";
  }
  const expires = new Date(token.expires_at);
  return expires.getTime() < Date.now()
    ? `Expired ${formatDate(token.expires_at)}`
    : formatDate(token.expires_at);
}
function scopeLabel(key: string): string {
  return scopeDescriptions.value.find((scope) => scope.key === key)?.name ?? key;
}

async function deleteToken(id: number) {
  await http
    .delete(`/api/user/token/delete/${id}`)
    .then(() => getAuthTokens())
    .catch((error) => console.error(error));
}
async function revokeAll() {
  if (
    !window.confirm(
      "Revoke every API token on your account? Anything using one — CI, a local `mvn` or `npm` — stops working immediately.",
    )
  ) {
    return;
  }
  await http
    .delete<{ revoked: number }>("/api/user/token/revoke-all")
    .then((response) => {
      notify({
        type: "success",
        title: "Tokens revoked",
        text: `${response.data.revoked} token(s) revoked.`,
      });
      getAuthTokens();
    })
    .catch((error) => {
      console.error(error);
      notify({
        type: "error",
        title: "Could not revoke tokens",
        text: "An error occurred while revoking your tokens.",
      });
    });
}
async function getAuthTokens() {
  if (user == undefined) {
    return;
  }
  await http
    .get<Array<RawAuthTokenFullResponse>>("/api/user/token/list")
    .then((response) => {
      authTokens.value = response.data;
    })
    .catch((error) => console.error(error));
}
async function getScopeDescriptions() {
  await http
    .get<Array<ScopeDescription>>("/api/info/scopes")
    .then((response) => {
      scopeDescriptions.value = response.data;
    })
    .catch((error) => console.error(error));
}
getAuthTokens();
getScopeDescriptions();
</script>

<style scoped lang="scss">
main {
  padding: 1rem;
}
.header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
}
.tokenList {
  list-style: none;
  padding: 0;
  margin: 0;
}

.tokenElement {
  border: 1px solid #000;
}
.tokenElementLine {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(10rem, 1fr));
  padding: 0.5rem;
  gap: 1rem;
  &:hover {
    cursor: pointer;
  }
}
.tokenDetail {
  padding: 0 0.5rem 0.5rem;
  h3 {
    margin-bottom: 0.25rem;
  }
}
.scopeList {
  margin: 0;
  padding-left: 1.25rem;
}
</style>
