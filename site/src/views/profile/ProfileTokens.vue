<template>
  <main>
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
            :value="new Date(token.token.created_at).toLocaleDateString()" />
        </div>
        <div v-if="expandedToken == token.token.id">
          <SubmitButton @click="deleteToken(token.token.id)">Delete</SubmitButton>
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
import { type RawAuthTokenFullResponse } from "@/types/user/token";
import { ref } from "vue";

const session = sessionStore();
const user = session.user;
const authTokens = ref<Array<RawAuthTokenFullResponse>>([]);
const expandedToken = ref<number | undefined>(undefined);
function tokenClicked(tokenId: number) {
  if (expandedToken.value == tokenId) {
    expandedToken.value = undefined;
  } else {
    expandedToken.value = tokenId;
  }
}
async function deleteToken(id: number) {
  await http
    .delete(`/api/user/token/delete/${id}`)
    .then(() => {
      getAuthTokens();
    })
    .catch((error) => {
      console.log(error);
    });
}
async function getAuthTokens() {
  if (user == undefined) {
    return;
  }

  await http
    .get<Array<RawAuthTokenFullResponse>>("/api/user/token/list")
    .then((response) => {
      console.log(response.data);
      authTokens.value = response.data;
    })
    .catch((error) => {
      console.log(error);
    });
}
getAuthTokens();
</script>

<style scoped lang="scss">
main {
  padding: 1rem;
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
  grid-template-columns: 1fr 1fr 1fr;
  padding: 0.5rem;
  gap: 1rem;
  &:hover {
    cursor: pointer;
  }
}
</style>
