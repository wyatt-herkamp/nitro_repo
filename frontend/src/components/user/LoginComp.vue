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
import httpCommon from "@/http-common";
import "@/styles/forms.css";
export default defineComponent({
  emits: ["login"],
  setup() {
    const form = ref({
      username: "",
      password: "",
    });
    return { form };
  },
  methods: {
    async onSubmit() {
      await httpCommon.apiClient
        .post("/api/login", this.form)
        .then((res) => {
          if (res.status === 200) {
            this.$emit("login", "success");
          }
        })
        .catch((err) => {
          if (err.response) {
            if (err.response.status === 401) {
              this.form.password = "";
              this.$emit("login", "failure");
            } else {
              console.error(err.response.data);
              this.$emit("login", "internal_error");
            }
          } else {
            console.error(err.request);
            this.$emit("login", "frontend_error");
          }
        });
    },
  },
});
</script>
<style scoped>
.loginButton {
  @apply bg-secondary;
  @apply text-quaternary;
  @apply py-2;
  @apply px-4;
  @apply rounded-md;
  @apply hover:bg-secondary/70;
}
</style>
