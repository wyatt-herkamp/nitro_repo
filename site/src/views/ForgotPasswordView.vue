<template>
  <main class="container container-narrow">
    <NCard title="Reset your password">
      <form
        v-if="!requested"
        class="stack"
        @submit.prevent="request">
        <p class="intro">
          Enter the email address on your account and we will send a link to set a new password.
        </p>

        <div class="field">
          <label for="resetEmail">Email address</label>
          <input
            id="resetEmail"
            v-model="email"
            type="email"
            required
            autocomplete="email"
            placeholder="you@example.com" />
        </div>

        <NButton
          type="submit"
          variant="primary"
          :loading="sending"
          block>
          Send reset link
        </NButton>
      </form>

      <!-- Deliberately the same message whether or not the address matched an account: the endpoint
           answers 200 either way so that this page cannot be used to enumerate registered emails,
           and saying "no such user" here would give that back. -->
      <div
        v-else
        class="sent">
        <font-awesome-icon
          icon="check-circle"
          class="sentIcon" />
        <p>
          If an account exists for <strong>{{ email }}</strong
          >, a reset link is on its way. The link expires, so use it soon.
        </p>
      </div>
    </NCard>

    <p class="back">
      <RouterLink :to="{ name: 'login' }">Back to sign in</RouterLink>
    </p>
  </main>
</template>

<script setup lang="ts">
/**
 * The login page has linked to `/forgot-password` all along, and the route did not exist — so the
 * link 404'd. The backend flow it needs has been there the whole time
 * (`POST /api/user/password-reset/request`).
 */
import { ref } from "vue";
import { RouterLink } from "vue-router";
import { notify } from "@kyvg/vue3-notification";
import NCard from "@/components/core/ui/NCard.vue";
import NButton from "@/components/core/ui/NButton.vue";
import http from "@/http";

const email = ref("");
const sending = ref(false);
const requested = ref(false);

async function request() {
  sending.value = true;
  try {
    await http.post("/api/user/password-reset/request", { email: email.value });
    requested.value = true;
  } catch {
    notify({
      type: "error",
      title: "Could not send the reset link",
      text: "Check the address and try again.",
    });
  } finally {
    sending.value = false;
  }
}
</script>

<style scoped lang="scss">
.intro {
  color: var(--text-muted);
  font-size: var(--text-sm);
}

.sent {
  display: flex;
  align-items: flex-start;
  gap: var(--space-3);
  color: var(--text-muted);
  font-size: var(--text-sm);

  strong {
    color: var(--text);
    font-weight: var(--weight-medium);
  }
}

.sentIcon {
  color: var(--success);
  margin-top: 0.15rem;
}

.back {
  margin-top: var(--space-4);
  text-align: center;
  font-size: var(--text-sm);
}
</style>
