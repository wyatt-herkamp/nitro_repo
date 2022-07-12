<template>
  <div>
    <form class="nitroForm" @submit.prevent="onSubmit()">
      <div class="formGroup">
        <label class="formLabel" for="username">Username</label>
        <input
          id="username"
          v-model="form.username"
          autocomplete="username"
          class="formInput"
          placeholder="Username"
          type="text"
        />
      </div>
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
      <div class="formGroup flex flex-row-reverse">
        <button class="loginButton">Sign in</button>
      </div>
    </form>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref } from "vue";
import { AuthToken, login } from "@nitro_repo/nitro_repo-api-wrapper";
import { useCookies } from "vue3-cookies";

export default defineComponent({
  emits: ["login"],
  setup() {
    const { cookies } = useCookies();

    const form = ref({
      username: "",
      password: "",
    });
    return { form, cookies };
  },
  methods: {
    async onSubmit() {
      const value = await login(this.form.username, this.form.password);
      if (value.ok) {
        const loginRequest = value.val as AuthToken;
        const date = new Date(loginRequest.expiration * 1000);
        this.cookies.set(
          "token",
          loginRequest.token,
          date,
          undefined,
          undefined,
          undefined,
          "Lax"
        );
        this.$emit("login", "success");
      } else {
        this.form.password = "";
        this.$emit("login", "failure");
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
