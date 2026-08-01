<template>
  <div class="stack">
    <div class="field">
      <label :for="`${id}-bucket`">Bucket name</label>
      <input
        :id="`${id}-bucket`"
        v-model="model.bucket_name"
        class="mono"
        required
        spellcheck="false"
        autocomplete="off" />
    </div>

    <div class="field">
      <label :for="`${id}-endpoint-mode`">Endpoint</label>
      <select
        :id="`${id}-endpoint-mode`"
        v-model="endpointMode">
        <option value="aws">An AWS region</option>
        <option value="custom">A custom endpoint (MinIO, R2, Ceph…)</option>
      </select>
    </div>

    <div
      v-if="endpointMode === 'aws'"
      class="field">
      <label :for="`${id}-region`">Region</label>
      <select
        :id="`${id}-region`"
        v-model="model.region">
        <option
          v-for="region in regions"
          :key="region"
          :value="region">
          {{ region }}
        </option>
      </select>
    </div>

    <template v-else>
      <div class="field">
        <label :for="`${id}-endpoint`">Endpoint URL</label>
        <input
          :id="`${id}-endpoint`"
          v-model="model.endpoint"
          class="mono"
          type="url"
          required
          placeholder="http://localhost:9000"
          spellcheck="false" />
      </div>

      <div class="field">
        <label :for="`${id}-custom-region`">Region name</label>
        <input
          :id="`${id}-custom-region`"
          v-model="model.custom_region"
          class="mono"
          placeholder="us-east-1"
          spellcheck="false" />
        <span class="field-hint">
          SigV4 needs a region in the credential scope even when the server ignores it. Left blank,
          it signs as <code>us-east-1</code>, which S3-compatible servers conventionally accept.
        </span>
      </div>
    </template>

    <div class="field">
      <label class="checkboxLabel">
        <input
          v-model="model.path_style"
          type="checkbox" />
        Path-style addressing
      </label>
      <span class="field-hint">
        <code>host/bucket/key</code> rather than <code>bucket.host/key</code>. Required by most
        self-hosted S3 servers; AWS itself wants this off.
      </span>
    </div>

    <fieldset class="credentials">
      <legend>Credentials</legend>
      <p class="field-hint">
        Leave both blank to use the AWS default provider chain — environment variables, the shared
        profile, an IAM role or web identity. That is the right choice on EC2, ECS and EKS.
      </p>

      <div class="credentialGrid">
        <div class="field">
          <label :for="`${id}-access-key`">Access key ID</label>
          <input
            :id="`${id}-access-key`"
            v-model="accessKey"
            class="mono"
            autocomplete="off"
            spellcheck="false" />
        </div>

        <div class="field">
          <label :for="`${id}-secret-key`">Secret access key</label>
          <input
            :id="`${id}-secret-key`"
            v-model="secretKey"
            class="mono"
            type="password"
            autocomplete="new-password"
            spellcheck="false" />
        </div>
      </div>
    </fieldset>
  </div>
</template>

<script setup lang="ts">
/**
 * S3 storage configuration.
 *
 * Phase 1 rewrote the S3 backend on the AWS SDK and added FileSystemV2, but `storageTypes` in the
 * frontend still listed only `Local` — so neither could be created through the UI at all.
 *
 * `custom_region` and `endpoint` are `#[serde(flatten)]`ed into `S3Config` on the server, so they
 * sit at the top level of the model rather than under a nested object.
 */
import { computed, ref, useId, watch } from "vue";
import http from "@/http";

const model = defineModel<Record<string, unknown>>({ required: true });

const id = useId();
const regions = ref<Array<string>>([]);
const endpointMode = ref<"aws" | "custom">(model.value.endpoint ? "custom" : "aws");

// The server treats a custom endpoint as taking precedence over `region`, so leaving a stale value
// in the field the user is not using would silently win.
watch(endpointMode, (mode) => {
  if (mode === "aws") {
    delete model.value.endpoint;
    delete model.value.custom_region;
    model.value.region ??= "UsEast1";
  } else {
    delete model.value.region;
    model.value.endpoint ??= "";
  }
});

// Credentials live under a nested object; these keep the inputs bound to it while treating an empty
// field as "unset" rather than as an empty key, which would fail to authenticate rather than
// falling back to the provider chain.
function credential(key: "access_key" | "secret_key") {
  return computed<string>({
    get: () => ((model.value.credentials as Record<string, string>)?.[key] ?? "") as string,
    set: (value) => {
      const credentials = (model.value.credentials ?? {}) as Record<string, string | undefined>;
      credentials[key] = value === "" ? undefined : value;
      model.value.credentials = credentials;
    },
  });
}

const accessKey = credential("access_key");
const secretKey = credential("secret_key");

async function loadRegions() {
  try {
    const response = await http.get<Array<string>>("/api/storage/s3/regions");
    regions.value = response.data;
  } catch {
    // The dropdown is a convenience; a custom endpoint does not need it.
    regions.value = [];
  }
}
loadRegions();
</script>

<style scoped lang="scss">
.checkboxLabel {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  color: var(--text);
}

.credentials {
  padding: var(--space-4);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);

  legend {
    padding-inline: var(--space-2);
    font-size: var(--text-xs);
    font-weight: var(--weight-semibold);
    letter-spacing: var(--tracking-label);
    text-transform: uppercase;
    color: var(--text-subtle);
  }
}

.credentialGrid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(14rem, 1fr));
  gap: var(--space-4);
  margin-top: var(--space-3);
}

code {
  background-color: var(--bg-sunken);
  padding: 0.0625rem 0.25rem;
  border-radius: var(--radius-sm);
}
</style>
