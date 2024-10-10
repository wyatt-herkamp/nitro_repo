<template>
  <main>
    <h1>Login Page</h1>
    <form @submit.prevent="login">
      <h4 v-if="failedLogin">Invalid username or password</h4>
      <TextInput
        id="username"
        v-model="input.email_or_username"
        autocomplete="username"
        autocapitalize="false"
        required
        autofocus
        placeholder="Username or Email">
        Username or Email
      </TextInput>
      <PasswordInput
        id="password"
        v-model="input.password"
        required
        >Password</PasswordInput
      >
      <div class="forgotPassword">
        <router-link to="/forgot-password">Forgot Password?</router-link>
      </div>
      <SubmitButton title="Login">Login</SubmitButton>
    </form>
  </main>
</template>
<script setup lang="ts">
import SubmitButton from "@/components/form/SubmitButton.vue";
import PasswordInput from "@/components/form/text/PasswordInput.vue";
import TextInput from "@/components/form/text/TextInput.vue";
import http from "@/http";
import router from "@/router";
import { sessionStore } from "@/stores/session";
import { notify } from "@kyvg/vue3-notification";
import { ref } from "vue";
const failedLogin = ref(false);
const input = ref({
  email_or_username: "",
  password: "",
});
const session = sessionStore();
async function login() {
  http
    .post("/api/user/login", input.value)
    .then((response) => {
      console.log(response);
      session.login(response.data);
      router.push("/");
    })
    .catch((error) => {
      if (error.response.status === 401) {
        failedLogin.value = true;
        notify({
          type: "error",
          title: "Login Failed",
          text: "Invalid username or password",
        });
      } else {
        console.log(error);
        notify({
          type: "error",
          title: "Login Failed",
          text: "An error occurred while trying to login",
        });
      }
    });
}
</script>
<style scoped lang="scss">
@import "@/assets/styles/theme.scss";
main {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100vh;
}
form {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}
.forgotPassword {
  text-align: right;
  color: $text;
  a {
    color: $text;
    text-decoration: none;
  }
}
</style>
