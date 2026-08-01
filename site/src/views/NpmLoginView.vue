<template>
  <main class="npmLogin">
    <h1>Authorize npm</h1>

    <p v-if="loading">Checking this login request…</p>

    <template v-else-if="error">
      <p class="error">{{ error }}</p>
      <p>
        The request may have expired — sessions are only valid for a short time. Run
        <code>npm login</code> again to start a new one.
      </p>
    </template>

    <template v-else-if="approved">
      <p class="approved">Approved. You can close this tab and return to your terminal.</p>
    </template>

    <template v-else-if="session">
      <p>
        <code>npm</code> is asking for a token to publish to
        <strong>{{ session.repository_name }}</strong
        >.
      </p>
      <p>
        Approving creates an access token scoped to that repository, with read and write permission.
        You can revoke it later from your profile.
      </p>
      <SubmitButton
        title="Authorize"
        @click="approve"
        >Authorize npm</SubmitButton
      >
    </template>
  </main>
</template>

<script setup lang="ts">
import SubmitButton from "@/components/form/SubmitButton.vue";
import http from "@/http";
import router from "@/router";
import { sessionStore } from "@/stores/session";
import { notify } from "@kyvg/vue3-notification";
import { onMounted, ref } from "vue";
import { useRoute } from "vue-router";

interface NpmLoginSession {
  repository_id: string;
  repository_name: string;
}

const route = useRoute();
const store = sessionStore();
const session = ref<NpmLoginSession | undefined>(undefined);
const loading = ref(true);
const approved = ref(false);
const error = ref<string | undefined>(undefined);

const sessionId = route.params.session as string;

onMounted(async () => {
  // Approving mints a token, so an anonymous visitor has to sign in first. The session id is
  // carried through the redirect so they land back here rather than on the home page.
  if (store.user === undefined) {
    router.push({ path: "/login", query: { redirect: route.fullPath } });
    return;
  }
  await http
    .get<NpmLoginSession>(`/api/npm/login/${sessionId}`)
    .then((response) => {
      session.value = response.data;
    })
    .catch(() => {
      error.value = "This login request is unknown or has expired.";
    })
    .finally(() => {
      loading.value = false;
    });
});

async function approve() {
  await http
    .post(`/api/npm/login/${sessionId}`, {})
    .then(() => {
      approved.value = true;
    })
    .catch((err) => {
      const status = err.response?.status;
      if (status === 403) {
        error.value = "You do not have permission to publish to this repository.";
      } else {
        error.value = "This login request is unknown or has expired.";
      }
      notify({
        type: "error",
        title: "Authorization failed",
        text: error.value,
      });
    });
}
</script>

<style lang="scss" scoped>
.npmLogin {
  max-width: 40rem;
  margin: 0 auto;
}
.error {
  color: #ff6b6b;
}
.approved {
  font-weight: bold;
}
</style>
