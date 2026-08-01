<template>
  <div class="stack">
    <div class="field">
      <label :for="`${id}-path`">Path</label>
      <input
        :id="`${id}-path`"
        v-model="model.path"
        class="mono"
        required
        spellcheck="false"
        autocomplete="off" />
      <span class="field-hint">
        A directory on the server, holding one sub-directory per repository.
      </span>
    </div>

    <div class="field">
      <label :for="`${id}-compression`">Compression</label>
      <select
        :id="`${id}-compression`"
        v-model="model.compression">
        <option value="None">None</option>
        <option value="Zstd">Zstandard</option>
        <option value="Gzip">Gzip</option>
      </select>
      <span class="field-hint">
        Shrinks text-ish artifacts — POMs, metadata XML, packuments. Jars and tarballs are already
        compressed, so there is little to gain on those.
      </span>
    </div>

    <div class="field">
      <label class="checkboxLabel">
        <input
          v-model="model.sync"
          type="checkbox" />
        Flush each object to disk before publishing it
      </label>
      <span class="field-hint">
        Costs a flush per write, in exchange for a completed upload surviving a power loss. The
        publishing rename is atomic either way, so without this a crash loses the write rather than
        corrupting anything.
      </span>
    </div>
  </div>
</template>

<script setup lang="ts">
/**
 * FileSystemV2 configuration.
 *
 * The backend has existed since Phase 1 and was unreachable from the UI, because `storageTypes`
 * listed only `Local`.
 */
import { useId } from "vue";

const model = defineModel<Record<string, unknown>>({ required: true });
const id = useId();
</script>

<style scoped lang="scss">
.checkboxLabel {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  color: var(--text);
}
</style>
