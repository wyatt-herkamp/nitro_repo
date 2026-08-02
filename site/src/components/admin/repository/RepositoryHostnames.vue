<template>
  <NCard
    title="Custom domains"
    subtitle="Serve this repository from its own hostname, with no /repositories prefix.">
    <div class="stack">
      <p class="hint">
        A request arriving on one of these hosts goes straight to this repository, and the whole
        request path is the artifact path —
        <code>https://{{ exampleHost }}/dev/kingtux/tms/…</code>. <code>/api</code>,
        <code>/badge</code> and <code>/repositories</code> keep working on every host, so an
        artifact path starting with one of those is only reachable through the repository's normal
        URL.
      </p>

      <NEmptyState
        v-if="!loading && hostnames.length === 0"
        icon="globe"
        title="No custom domains"
        description="This repository is reachable through its storage and repository name only." />

      <ul
        v-else-if="hostnames.length > 0"
        class="hostnames">
        <li
          v-for="hostname in hostnames"
          :key="hostname.id">
          <span class="mono">{{ hostname.hostname }}</span>
          <NButton
            variant="ghost"
            icon="trash"
            size="sm"
            :disabled="removing !== undefined"
            @click="pendingRemoval = hostname">
            Remove
          </NButton>
        </li>
      </ul>

      <form
        class="add"
        @submit.prevent="add">
        <div class="field">
          <label for="newHostname">Add a domain</label>
          <input
            id="newHostname"
            v-model="newHostname"
            class="mono"
            placeholder="maven.example.com"
            spellcheck="false"
            autocomplete="off" />
        </div>
        <NButton
          type="submit"
          variant="primary"
          :loading="adding"
          :disabled="newHostname.trim().length === 0">
          Add domain
        </NButton>
      </form>

      <p
        v-if="validationError"
        class="error">
        {{ validationError }}
      </p>

      <p class="hint">
        DNS and TLS for the domain have to point at this instance already — adding it here only
        tells nitro-repo what to do with the request once it arrives. Behind a reverse proxy, the
        original <code>Host</code> must be forwarded (nginx:
        <code>proxy_set_header Host $host;</code>), or every request will arrive under the proxy's
        own hostname and none of this will match.
      </p>
    </div>

    <NConfirmDialog
      :model-value="pendingRemoval !== undefined"
      title="Remove this domain?"
      :message="`Requests to ${pendingRemoval?.hostname} will stop reaching this repository. Anything pulling from that host will break.`"
      confirm-label="Remove domain"
      destructive
      :loading="removing !== undefined"
      @update:model-value="(open: boolean) => !open && (pendingRemoval = undefined)"
      @confirm="remove" />
  </NCard>
</template>

<script setup lang="ts">
/**
 * Custom domains for a repository.
 *
 * Rendered inside the Main tab rather than as a tab of its own: the tab strip is built from the
 * repository's config keys, and a hostname is not a config — it is a row in a table with an
 * instance-wide unique constraint.
 */
import { computed, ref } from "vue";
import { notify } from "@kyvg/vue3-notification";
import http from "@/http";
import NCard from "@/components/core/ui/NCard.vue";
import NButton from "@/components/core/ui/NButton.vue";
import NEmptyState from "@/components/core/ui/NEmptyState.vue";
import NConfirmDialog from "@/components/core/ui/NConfirmDialog.vue";
import type { RepositoryHostname } from "@/types/repository";

const props = defineProps<{ repositoryId: string }>();

const hostnames = ref<Array<RepositoryHostname>>([]);
const newHostname = ref("");
const validationError = ref<string | undefined>(undefined);
const loading = ref(true);
const adding = ref(false);
const removing = ref<number | undefined>(undefined);
const pendingRemoval = ref<RepositoryHostname | undefined>(undefined);

const exampleHost = computed(() => hostnames.value[0]?.hostname ?? "maven.example.com");

async function load() {
  loading.value = true;
  try {
    const response = await http.get<Array<RepositoryHostname>>(
      `/api/repository/${props.repositoryId}/hostnames`,
    );
    hostnames.value = response.data;
  } catch {
    notify({ type: "error", title: "Could not load this repository's domains" });
  } finally {
    loading.value = false;
  }
}

/**
 * Catches the mistakes that do not need a round trip — a pasted URL, mostly. The server is still
 * the authority on what a valid hostname is.
 */
function localProblem(hostname: string): string | undefined {
  if (/^[a-z]+:\/\//i.test(hostname)) {
    return "Enter just the hostname, without https:// in front.";
  }
  if (hostname.includes("/")) {
    return "Enter just the hostname, without a path.";
  }
  if (hostname.includes(":")) {
    return "Enter just the hostname, without a port.";
  }
  return undefined;
}

async function add() {
  const hostname = newHostname.value.trim().toLowerCase();
  validationError.value = localProblem(hostname);
  if (validationError.value) return;

  adding.value = true;
  try {
    const response = await http.post<RepositoryHostname>(
      `/api/repository/${props.repositoryId}/hostnames`,
      { hostname },
    );
    hostnames.value.push(response.data);
    newHostname.value = "";
    notify({ type: "success", title: `${hostname} now serves this repository` });
  } catch (error: unknown) {
    const response = (error as { response?: { status?: number; data?: unknown } })?.response;
    if (response?.status === 409) {
      validationError.value =
        "That domain is already in use, or it is the hostname this instance itself is served on.";
    } else if (response?.status === 400) {
      validationError.value =
        typeof response.data === "string" ? response.data : "That is not a valid hostname.";
    } else {
      notify({ type: "error", title: "Could not add the domain" });
    }
  } finally {
    adding.value = false;
  }
}

async function remove() {
  const hostname = pendingRemoval.value;
  if (!hostname) return;

  removing.value = hostname.id;
  try {
    await http.delete(`/api/repository/${props.repositoryId}/hostnames/${hostname.id}`);
    hostnames.value = hostnames.value.filter((entry) => entry.id !== hostname.id);
    notify({ type: "success", title: `${hostname.hostname} removed` });
  } catch {
    notify({ type: "error", title: "Could not remove the domain" });
  } finally {
    removing.value = undefined;
    pendingRemoval.value = undefined;
  }
}

load();
</script>

<style scoped lang="scss">
.hostnames {
  list-style: none;
  margin: 0;
  padding: 0;
  border: 1px solid var(--border);
  border-radius: var(--radius-md);

  li {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-4);
    padding: var(--space-2) var(--space-3);

    & + li {
      border-top: 1px solid var(--border);
    }
  }
}

.add {
  display: flex;
  align-items: flex-end;
  gap: var(--space-3);
  flex-wrap: wrap;

  .field {
    flex: 1 1 18rem;
  }
}

.hint {
  max-width: 42rem;
  font-size: var(--text-sm);
  color: var(--text-muted);
}

.error {
  font-size: var(--text-sm);
  color: var(--danger);
}
</style>
