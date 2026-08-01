<template>
  <TabsElement
    v-if="user"
    default-tab="main"
    :justify-between="false">
    <template #header>
      <TabElement id="main">User</TabElement>
      <TabElement id="password">Password</TabElement>
      <TabElement id="user-permissions">Permissions</TabElement>
      <TabElement id="repository-permissions">Repository access</TabElement>
    </template>

    <template #content>
      <TabContent tabId="main">
        <div class="mainGrid">
          <NCard title="Details">
            <form
              class="stack"
              @submit.prevent="saveUser">
              <TextInput
                id="name"
                v-model="changeUser.name"
                autocomplete="name">
                Name
              </TextInput>
              <ValidatableTextBox
                id="email"
                v-model="changeUser.email"
                autocomplete="email"
                :validations="EMAIL_VALIDATIONS"
                :originalValue="user.email">
                Email
              </ValidatableTextBox>
              <ValidatableTextBox
                id="username"
                v-model="changeUser.username"
                :originalValue="user.username"
                :validations="USERNAME_VALIDATIONS"
                :deniedKeys="[' ']"
                autocomplete="username">
                Username
              </ValidatableTextBox>
              <SubmitButton :disabled="savingUser">Save</SubmitButton>
            </form>
          </NCard>

          <NCard title="Account">
            <dl class="facts">
              <div>
                <dt>User ID</dt>
                <dd class="mono">{{ user.id }}</dd>
              </div>
              <div>
                <dt>Joined</dt>
                <dd>{{ formatDate(user.created_at) }}</dd>
              </div>
            </dl>
          </NCard>
        </div>
      </TabContent>

      <TabContent tabId="password">
        <NCard
          title="Set a new password"
          subtitle="The user is not notified. Tell them out of band.">
          <form
            class="stack passwordForm"
            @submit.prevent="changePassword">
            <!-- Hidden, but present so password managers can associate the new password with the
                 right account rather than offering to save it against the admin's own. -->
            <input
              type="hidden"
              name="email"
              autocomplete="email"
              :value="user.email" />
            <input
              type="hidden"
              name="username"
              autocomplete="username"
              :value="user.username" />
            <NewPasswordInput
              v-if="passwordRules"
              id="password"
              v-model="newPassword"
              :passwordRules="passwordRules">
              Password
            </NewPasswordInput>
            <SubmitButton :disabled="changingPassword">Save</SubmitButton>
          </form>
        </NCard>
      </TabContent>

      <TabContent tabId="user-permissions">
        <UserPermissions :user="user" />
      </TabContent>

      <TabContent tabId="repository-permissions">
        <RepositoryPermissions :user="user" />
      </TabContent>
    </template>
  </TabsElement>
</template>

<script setup lang="ts">
/**
 * The admin's view of one user.
 *
 * This carried a second, independent tabs implementation — hand-rolled clickable `div`s with their
 * own stylesheet, alongside the shared `core/tabs/` components used everywhere else. It is on the
 * shared ones now, which are keyboard-reachable and announced as tabs.
 *
 * The details form also had no `@submit` handler and no endpoint behind it, so a name, username or
 * email could never actually be changed. `PUT /api/user-management/update/{id}` now exists.
 */
import SubmitButton from "@/components/form/SubmitButton.vue";
import NewPasswordInput from "@/components/form/text/NewPasswordInput.vue";
import TextInput from "@/components/form/text/TextInput.vue";
import ValidatableTextBox from "@/components/form/text/ValidatableTextBox.vue";
import TabsElement from "@/components/core/tabs/TabsElement.vue";
import TabElement from "@/components/core/tabs/TabElement.vue";
import TabContent from "@/components/core/tabs/TabContent.vue";
import NCard from "@/components/core/ui/NCard.vue";
import { siteStore } from "@/stores/site";
import type { UserResponseType } from "@/types/base";
import { ref, type PropType } from "vue";
import UserPermissions from "./UserPermissions.vue";
import RepositoryPermissions from "./RepositoryPermissions.vue";
import http from "@/http";
import { notify } from "@kyvg/vue3-notification";
import { EMAIL_VALIDATIONS, USERNAME_VALIDATIONS } from "@/components/form/text/validations";
import { formatDate } from "@/utils/format";

const props = defineProps({
  user: {
    type: Object as PropType<UserResponseType>,
    required: true,
  },
});

const changeUser = ref({
  name: props.user.name,
  email: props.user.email,
  username: props.user.username,
});
const newPassword = ref("");
const savingUser = ref(false);
const changingPassword = ref(false);

const passwordRules = siteStore().siteInfo?.password_rules;

async function saveUser() {
  savingUser.value = true;
  try {
    await http.put(`/api/user-management/update/${props.user.id}`, changeUser.value);
    notify({ type: "success", title: "User updated" });
  } catch (caught: unknown) {
    const status = (caught as { response?: { status?: number } })?.response?.status;
    notify({
      type: "error",
      title: status === 409 ? "That username or email is taken" : "Could not update the user",
    });
  } finally {
    savingUser.value = false;
  }
}

async function changePassword() {
  if (!newPassword.value) {
    return;
  }
  changingPassword.value = true;
  try {
    await http.put(`/api/user-management/update/${props.user.id}/password`, {
      password: newPassword.value,
    });
    notify({
      type: "success",
      title: "Password changed",
      text: "The user is not notified; tell them yourself.",
    });
    newPassword.value = "";
  } catch {
    notify({ type: "error", title: "Could not change the password" });
  } finally {
    changingPassword.value = false;
  }
}
</script>

<style scoped lang="scss">
.mainGrid {
  display: grid;
  grid-template-columns: minmax(0, 2fr) minmax(0, 1fr);
  gap: var(--space-4);
  align-items: start;

  @media (max-width: 56rem) {
    grid-template-columns: 1fr;
  }
}

.passwordForm {
  max-width: 26rem;
}

.facts {
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
  margin: 0;

  dt {
    font-size: var(--text-xs);
    font-weight: var(--weight-semibold);
    letter-spacing: var(--tracking-label);
    text-transform: uppercase;
    color: var(--text-subtle);
  }

  dd {
    margin: var(--space-1) 0 0;
    font-size: var(--text-sm);
  }
}
</style>
