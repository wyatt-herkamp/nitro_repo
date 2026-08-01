<template>
  <div class="field">
    <label for="localPath">Path</label>
    <input
      id="localPath"
      v-model="model.path"
      class="mono"
      required
      spellcheck="false"
      autocomplete="off" />
    <span class="field-hint">
      Changing this does not move any files. Point it elsewhere only after relocating the directory
      yourself.
    </span>
  </div>
</template>

<script setup lang="ts">
// The field carried `disabled`, because the page had no way to save anything —
// `PUT /api/storage/{id}` did not exist. It does now, so this is editable, with the warning that
// matters.
const model = defineModel<Record<string, unknown>>({ required: true });
</script>
