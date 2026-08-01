<template>
  <main class="loginPage">
    <div class="loginCard">
      <div class="brand">
        <img
          src="/icon-128.png"
          alt=""
          width="32"
          height="32" />
        <h1>Sign in</h1>
      </div>

      <form
        class="stack"
        @submit.prevent="login">
        <p
          v-if="failedLogin"
          class="field-error"
          role="alert">
          Invalid username or password.
        </p>

        <TextInput
          id="username"
          v-model="input.email_or_username"
          autocomplete="username"
          autocapitalize="off"
          required
          autofocus
          placeholder="Username or email">
          Username or email
        </TextInput>

        <PasswordInput
          id="password"
          v-model="input.password"
          required
          >Password</PasswordInput
        >

        <SubmitButton title="Sign in">Sign in</SubmitButton>

        <RouterLink
          class="forgotPassword"
          :to="{ name: 'forgotPassword' }">
          Forgot your password?
        </RouterLink>
      </form>
    </div>
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
import { RouterLink, useRoute } from "vue-router";

const failedLogin = ref(false);
const input = ref({
  email_or_username: "",
  password: "",
});
const session = sessionStore();
const route = useRoute();

async function login() {
  failedLogin.value = false;
  try {
    const response = await http.post("/api/user/login", input.value);
    session.login(response.data);
    // Anything that redirects here to sign in — the npm browser login, for one — needs the user to
    // come back rather than land on the home page. Only a relative path is honoured, so a crafted
    // `?redirect=https://…` cannot bounce someone off-site after they authenticate.
    const redirect = route.query.redirect;
    const target =
      typeof redirect === "string" && redirect.startsWith("/") && !redirect.startsWith("//")
        ? redirect
        : "/";
    router.push(target);
  } catch (caught: unknown) {
    // Reading `error.response.status` directly threw on a network failure, where there is no
    // response — so an unreachable server produced an unhandled rejection and no message at all.
    const status = (caught as { response?: { status?: number } })?.response?.status;
    if (status === 401) {
      failedLogin.value = true;
    } else {
      notify({
        type: "error",
        title: "Could not sign in",
        text: status === undefined ? "Could not reach the server." : "Something went wrong.",
      });
    }
  }
}
</script>

<style scoped lang="scss">
.loginPage {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: var(--space-8) var(--space-4);
}

.loginCard {
  width: 100%;
  max-width: 24rem;
  padding: var(--space-8);
  background-color: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-md);
}

.brand {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  margin-bottom: var(--space-6);

  h1 {
    font-size: var(--text-xl);
  }
}

.forgotPassword {
  text-align: center;
  font-size: var(--text-sm);
  color: var(--text-muted);

  &:hover {
    color: var(--accent);
  }
}
</style>
