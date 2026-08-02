<template>
  <div v-if="isCreate">
    <form @submit.prevent="">
      <DropDown
        v-model="input.cargoTypes"
        :options="cargoTypes"
        required
        >Cargo Registry Type</DropDown
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
      A hosted Cargo registry has nothing else to configure. The registry type cannot be changed
      after the repository is created.
    </p>
  </div>
</template>
<script setup lang="ts">
import DropDown from "@/components/form/dropdown/DropDown.vue";
import TextInput from "@/components/form/text/TextInput.vue";
import { type CargoConfigType } from "./cargo";
import http from "@/http";
import { computed, ref, watch } from "vue";
import { notify } from "@kyvg/vue3-notification";

// Only what `CargoRegistryConfig` on the backend accepts. Offering a variant the server does not
// have produces a repository it refuses to validate, which is how npm's `Proxy` option behaved.
const cargoTypes = [
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
  cargoTypes: "",
});
const isCreate = computed(() => {
  return !props.repository;
});
const value = defineModel<CargoConfigType>();
watch(input.value, () => {
  if (input.value.cargoTypes === "") {
    return;
  }
  if (!isCreate.value) {
    return;
  }
  if (input.value.cargoTypes === "Hosted") {
    value.value = {
      type: "Hosted",
    };
  } else {
    notify({
      type: "error",
      title: "Error",
      text: "Invalid Cargo registry type",
    });
    input.value.cargoTypes = "";
  }
});
async function load() {
  if (props.repository) {
    await http
      .get(`/api/repository/${props.repository}/config/cargo`)
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
