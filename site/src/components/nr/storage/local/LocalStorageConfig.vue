<template>
  <div class="field">
    <label for="newLocalPath">Path</label>
    <input
      id="newLocalPath"
      v-model="model.path"
      class="mono"
      required
      spellcheck="false"
      autocomplete="off" />
    <span class="field-hint">
      A directory on the server, holding one sub-directory per repository.
    </span>
  </div>
</template>

<script setup lang="ts">
import http from "@/http";

const model = defineModel<Record<string, unknown>>({ required: true });

// The server suggests a location under its data directory, so the field is not blank to start with.
async function suggestPath() {
  try {
    const response = await http.post<{ value: string }>("/api/storage/local/path-helper", {});
    if (!model.value.path) {
      model.value.path = response.data.value;
    }
  } catch {
    // Only a convenience; the field is still typeable.
  }
}
suggestPath();
</script>
