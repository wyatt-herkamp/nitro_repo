<template>
  <div v-if="isCreate">
    <form @submit.prevent="">
      <DropDown
        v-model="input.npmTypes"
        :options="npmTypes"
        required
        >NPM Type</DropDown
      >
    </form>
  </div>
  <div v-else-if="value">
    <TextInput
      v-model="value.type"
      required
      disabled
      >Repository Type</TextInput
    >
    <p class="note">
      A hosted npm registry has nothing else to configure. The registry type cannot be changed after
      the repository is created.
    </p>
  </div>
</template>
<script setup lang="ts">
import DropDown from "@/components/form/dropdown/DropDown.vue";
import TextInput from "@/components/form/text/TextInput.vue";
import { type NPMConfigType } from "./npm";
import http from "@/http";
import { computed, ref, watch } from "vue";
import { notify } from "@kyvg/vue3-notification";

// `Proxy` was offered here, but `NPMRegistryConfig` on the backend has only a `Hosted` variant, so
// choosing it produced a repository the server refused to validate. It comes back when the npm
// proxy does.
const npmTypes = [
  {
    value: "Hosted",
    label: "Hosted",
  },
];
const props = defineProps({
  settingName: String,
  repository: {
    type: String,
    required: false,
  },
});
const input = ref({
  npmTypes: "",
});
const isCreate = computed(() => {
  return !props.repository;
});
const value = defineModel<NPMConfigType>();
watch(input.value, () => {
  if (input.value.npmTypes === "") {
    return;
  }
  if (!isCreate.value) {
    return;
  }
  if (input.value.npmTypes === "Hosted") {
    value.value = {
      type: "Hosted",
    };
  } else {
    notify({
      type: "error",
      title: "Error",
      text: "Invalid npm registry type",
    });
    input.value.npmTypes = "";
  }
});
async function load() {
  if (props.repository) {
    await http
      .get(`/api/repository/${props.repository}/config/npm`)
      .then((response) => {
        value.value = response.data;
      })
      .catch((error) => {
        console.error(error);
      });
  }
}
load();
</script>
<style scoped lang="scss">
.note {
  opacity: 0.8;
  font-size: 0.9rem;
}
</style>
