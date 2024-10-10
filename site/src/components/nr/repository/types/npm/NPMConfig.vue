<template>
  <div v-if="!repository">
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
    <form @submit.prevent="save">
      <TextInput
        v-model="value.type"
        required
        disabled
        >Repository Type</TextInput
      >
    </form>
  </div>
</template>
<script setup lang="ts">
import DropDown from "@/components/form/dropdown/DropDown.vue";
import TextInput from "@/components/form/text/TextInput.vue";
import http from "@/http";
import { defaultProxy, type NPMConfigType } from "./npm";
import { computed, defineProps, ref, watch } from "vue";
import { notify } from "@kyvg/vue3-notification";

const npmTypes = [
  {
    value: "Hosted",
    label: "Hosted",
  },
  {
    value: "Proxy",
    label: "Proxy",
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
  console.log(input.value);

  if (input.value.npmTypes === "") {
    return;
  }
  if (isCreate.value) {
    if (input.value.npmTypes === "Hosted") {
      console.log("Setting Hosted");
      value.value = {
        type: "Hosted",
      };
    } else if (input.value.npmTypes === "Proxy") {
      value.value = {
        type: "Proxy",
        config: defaultProxy(),
      };
    } else {
      notify({
        type: "error",
        title: "Error",
        text: "Invalid maven type",
      });
      input.value.npmTypes = "";
    }
  }
  console.log(value.value);
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
async function save() {
  if (props.repository) {
    await http
      .put(`/api/repository/${props.repository}/config/npm`, value.value)
      .then(() => {
        console.log("Saved");
      })
      .catch((error) => {
        console.error(error);
      });
  }
}
</script>
