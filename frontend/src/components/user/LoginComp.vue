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
        <label class="nitroLabel" for="username">Password</label>
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
import { useRouter } from "vue-router";
import { login } from "nitro_repo-api-wrapper";
import { AuthToken } from "nitro_repo-api-wrapper";

export default defineComponent({
  setup() {
    let form = ref({
      username: "",
      password: "",
    });
    return { form };
  },
  methods: {
    async onSubmit(username: string, password: string) {
      const value = await login(username, password);
      if (value.ok) {
        let loginRequest = value.val as AuthToken;
        let date = new Date(loginRequest.expiration * 1000);
        this.$cookie.setCookie("token", loginRequest.token, {
          expire: date,
          sameSite: "lax",
        });
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
