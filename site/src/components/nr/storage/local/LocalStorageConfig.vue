<template>
  <div>
    <TextInput
      id="id"
      v-model="model.path"
      required
      autocomplete="none"
      spellcheck="false">
      Path
    </TextInput>
  </div>
</template>
<script setup lang="ts">
import TextInput from "@/components/form/text/TextInput.vue";
import http from "@/http";
const model = defineModel<any>();

async function getDefaultPath() {
  await http.post("/api/storage/local-storage-path-helper", {}).then((response) => {
    model.value.path = response.data.value;
  });
}
getDefaultPath();
</script>
