<template>
  <main v-if="!newResponseTokenResponse">
    <h1>Token Create</h1>
    <form @submit.prevent="createToken">
      <div id="regularProperties">
        <div class="column">
          <TextInput
            id="tokenName"
            v-model="newToken.tokenName"
            >Token Name</TextInput
          >
          <TextInput
            id="tokenDescription"
            v-model="newToken.tokenDescription">
            Token Description
          </TextInput>
        </div>
        <div class="column">
          <NumberInput
            id="tokenExpiration"
            v-model="newToken.expiresInDays"
            :min="1"
            placeholder="Leave empty for a token that never expires"
            >Expires after (days)</NumberInput
          >
        </div>
      </div>
      <h2>Repository Scopes</h2>
      <div>
        <RepositoryToActionList v-model="repositoryScopes" />
      </div>
      <h2>Scopes</h2>
      <div>
        <ScopesSelector v-model="scopes" />
      </div>
      <SubmitButton>Create Token</SubmitButton>
    </form>
  </main>
  <main v-else>
    <CopyCode :code="newResponseTokenResponse.token"
      >Your token. Copy it now — it is not shown again.</CopyCode
    >
  </main>
</template>
<script setup lang="ts">
import CopyCode from "@/components/core/code/CopyCode.vue";
import ScopesSelector from "@/components/form/ScopesSelector.vue";
import SubmitButton from "@/components/form/SubmitButton.vue";
import TextInput from "@/components/form/text/TextInput.vue";
import NumberInput from "@/components/form/NumberInput.vue";
import RepositoryToActionList from "@/components/nr/repository/RepositoryToActionList.vue";
import http from "@/http";
import type { ScopeDescription } from "@/types/base";
import type { RepositoryToActions } from "@/types/repository";
import {
  type NewAuthTokenRepositoryScopeRequest,
  type NewAuthTokenResponse,
} from "@/types/user/token";
import { notify } from "@kyvg/vue3-notification";
import { ref } from "vue";
const newToken = ref<{
  tokenName: string;
  tokenDescription: string;
  expiresInDays: number | undefined;
}>({
  tokenName: "",
  tokenDescription: "",
  expiresInDays: undefined,
});
const newResponseTokenResponse = ref<NewAuthTokenResponse | undefined>(undefined);
const repositoryScopes = ref<Array<RepositoryToActions>>([]);
const scopes = ref<Array<ScopeDescription>>([]);
async function createToken() {
  // `repository_string`/`actions` are not the field names the API reads — it wants
  // `repository_id`/`scopes` — so a token created with any repository scope was rejected outright.
  const repositoryScopesRequest: Array<NewAuthTokenRepositoryScopeRequest> =
    repositoryScopes.value.map((repositoryScope) => ({
      repository_id: repositoryScope.repositoryId,
      scopes: repositoryScope.actions.asArray(),
    }));
  const scopesRequest = scopes.value.map((scope) => scope.key);
  const request = {
    name: newToken.value.tokenName,
    description: newToken.value.tokenDescription,
    repository_scopes: repositoryScopesRequest,
    scopes: scopesRequest,
    expires_in_days: newToken.value.expiresInDays,
  };
  await http
    .post<NewAuthTokenResponse>("/api/user/token/create", request)
    .then((response) => {
      newResponseTokenResponse.value = response.data;
      notify({
        type: "success",
        title: "Token Created",
        text: "The token has been created.",
      });
    })
    .catch((error) => {
      console.error(error);
      notify({
        type: "error",
        title: "Error Creating Token",
        text: "An error occurred while creating the token.",
      });
    });
}
</script>

<style scoped lang="scss">
@import "@/assets/styles/theme";

#repositoryScopes {
  .row {
    display: grid;
    grid-template-columns: 1fr 1fr 1fr 1fr 1fr;
  }
  .actionButton {
    background-color: $primary;
    color: white;
    border: none;
    padding: 0.5rem;
    border-radius: 0.5rem;
    cursor: pointer;
    &:disabled {
      background-color: $primary-50;
      cursor: not-allowed;
    }
  }
  #header {
    border-bottom: 1px solid $primary-50;
    padding: 1rem 0rem;
    .col {
      font-weight: bold;
    }
  }
  .row {
    padding: 1rem;
    padding-top: 0.5rem;
  }
}
#regularProperties {
  display: flex;
  gap: 1rem;
  .column {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }
}
</style>
