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
import { useCookies } from "vue3-cookies";
import httpCommon from "@/http-common";

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
      await httpCommon.apiClient
        .post("/api/login", {
          username: this.form.username,
          password: this.form.password,
        })
        .then((response) => {
          if (response.status == 200) {
            this.$emit("login", "success");
          } else {
            this.$emit("login", "fail");
          }
        })
        .catch((error) => {
          console.error(error);
          this.$emit("login", "fail");
        });
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
