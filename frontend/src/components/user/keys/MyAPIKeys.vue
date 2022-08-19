<template>
  <APIKeys v-model="data" @newToken="newToken = true" @refresh="refresh()" />

  <vue-final-modal
    v-model="newToken"
    classes="flex justify-center items-center"
    @click-outside="exitModal()"
  >
    <div v-if="this.form.newToken.id === ''" class="modal">
      <div class="flex flex-row justify-between">
        <span class="header">Create New Token</span>
        <button class="xButton" @click="exitModal()">🗙</button>
      </div>

      <div>
        <form class="nitroForm" @submit.prevent="createNewToken">
          <div class="formGroup">
            <label class="formLabel" for="password">Password</label>
            <input
              id="password"
              v-model="form.password"
              autocomplete="current-password"
              class="formInput"
              placeholder="Password"
              type="password"
            />
          </div>
          <label class="formLabel" for="t-desc">Token Description</label>
          <input
            id="password"
            v-model="form.description"
            class="formInput"
            placeholder="My Cool Token"
            type="text"
          />

          <button
            class="bg-green-500 hover:bg-green-700 text-white font-bold py-2 px-4 h-12 rounded mt-2 w-full"
            type="submit"
          >
            Create new Token
          </button>
        </form>
      </div>
    </div>
    <div v-else class="modal text-quaternary">
      <div class="flex flex-row justify-between">
        <span class="header">New Token Created</span>
        <button class="xButton" @click="newToken = false">🗙</button>
      </div>

      <h2 class="text-quaternary text-left my-2">Token ID</h2>
      <code class="text-black select-text bg-white p-2">
        {{ form.newToken.id }}
      </code>
      <h2 class="text-quaternary text-left mt-2">Token</h2>
      <h4 class="text-quaternary text-left mb-2">
        Copy now or loose it forever
      </h4>
      <code class="text-black select-text bg-white p-2">
        {{ form.newToken.token }}
      </code>
    </div>
  </vue-final-modal>
</template>
<script lang="ts">
import { computed, defineComponent, ref } from "vue";
import httpCommon from "@/http-common";
import { AuthToken } from "@/types/userTypes";
import "@/styles/form.css";
import { useUserStore } from "@/store/user";

export default defineComponent({
  name: "MyAPIKeys",
  setup() {
    const newToken = ref(false);
    const data = ref<Array<AuthToken>>([]);
    const form = ref({
      password: "",
      description: "",
      newToken: {
        id: "",
        token: "",
      },
    });
    httpCommon.apiClient
      .get<Array<AuthToken>>("api/token/list")
      .then((response) => {
        data.value = response.data;
      });
    const userStore = useUserStore();

    const user = computed(() => {
      return userStore.$state.user;
    });
    return { data, newToken, form, user };
  },
  methods: {
    exitModal() {
      this.newToken = false;
      this.form = {
        password: "",
        description: "",
        newToken: {
          id: "",
          token: "",
        },
      };
    },
    async refresh() {
      httpCommon.apiClient
        .get<Array<AuthToken>>("api/token/list")
        .then((response) => {
          this.data = response.data;
        });
    },
    async createNewToken() {
      if (this.user != undefined) {
        await httpCommon.apiClient
          .post<{ token_id: string; token: string }>("api/token/create", {
            username: this.user.username,
            password: this.form.password,
            secure_data: this.form.description,
          })
          .then((response) => {
            if (response.status === 200 || response.status === 201) {
              this.form.newToken.token = response.data.token;
              this.form.newToken.id = response.data.token_id;
            } else {
              this.$notify({
                type: "error",
                text: "Error creating token",
              });
            }
          });
        httpCommon.apiClient
          .get<Array<AuthToken>>("api/token/list")
          .then((response) => {
            this.data = response.data;
          });
      }
    },
  },
});
</script>
<style scoped>
.header {
  @apply font-bold;
  @apply text-xl;
  @apply text-quaternary;
  @apply min-w-fit;
}

.modal {
  @apply relative;
  @apply border;
  @apply bg-primary;
  @apply border-primary/90;
  @apply py-5;
  @apply px-10;
  @apply rounded-2xl;
  @apply shadow-xl;
  @apply text-center;
  @apply min-h-full;
}
</style>
