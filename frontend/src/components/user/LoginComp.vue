<template>
  <div>
    <form
      class="nitroForm"
      @submit.prevent="onSubmit(form.username, form.password)"
    >
      <div class="py-2">
        <label class="nitroLabel" for="username">Username</label>
        <input
          id="username"
          v-model="form.username"
          autocomplete="username"
          class="nitroTextInput"
          placeholder="Username"
          type="text"
        />
      </div>
      <div class="py-2">
        <label class="nitroLabel" for="password">Password</label>
        <input
          id="password"
          v-model="form.password"
          autocomplete="current-password"
          class="nitroTextInput"
          placeholder="Password"
          type="password"
        />
      </div>
      <div class="py-2">
        <button class="loginButton nitroButton">Sign in</button>
      </div>
    </form>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref } from "vue";
import { login } from "@nitro_repo/nitro_repo-api-wrapper";
import { AuthToken } from "@nitro_repo/nitro_repo-api-wrapper";
import { useCookies } from "vue3-cookies";

export default defineComponent({
  setup() {
    const { cookies } = useCookies();

    let form = ref({
      username: "",
      password: "",
    });
    return { form, cookies };
  },
  methods: {
    async onSubmit(username: string, password: string) {
      const value = await login(username, password);
      if (value.ok) {
        let loginRequest = value.val as AuthToken;
        let date = new Date(loginRequest.expiration * 1000);
        this.cookies.set(
          "token",
          loginRequest.token,
          date,
          undefined,
          undefined,
          undefined,
          "Lax"
        );
        location.reload();
      } else {
        this.form.password = "";
        this.$notify({
          title: value.val.user_friendly_message,
          type: "warn",
        });
      }
    },
  },
});
</script>
<style>
.loginButton:hover {
  @apply bg-slate-900;
  transition: background-color 0.5s;
}
</style>
