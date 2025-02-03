<template>
  <main>
    <h1>Repository Create</h1>
    <div v-if="currentRepositoryType">
      <h2>{{ currentRepositoryType.description }}</h2>
    </div>

    <form @submit.prevent="createRepository()">
      <TwoByFormBox>
        <TextInput
          id="repositoryName"
          v-model="input.name"
          autocomplete="none"
          required
          placeholder="Repository Name"
          >Repository Name</TextInput
        >
        <DropDown
          id="repositoryType"
          v-model="selectedRepositoryType"
          :options="repositoryTypeOptions"
          required
          >Repository Type</DropDown
        >
        <DropDown
          id="storage"
          v-model="input.storage"
          :options="storageItemOptions"
          required
          >Storage</DropDown
        >
      </TwoByFormBox>
      <div
        v-for="config in requiredConfigComponents"
        :key="config.component.name">
        <component
          :is="config.component"
          v-bind="config.props"
          v-model="requiredConfigValues[config.configName]" />
      </div>

      <SubmitButton>Create</SubmitButton>
    </form>
  </main>
</template>

<script lang="ts" setup>
import FallBackEditor from "@/components/admin/repository/configs/FallBackEditor.vue";
import DropDown from "@/components/form/dropdown/DropDown.vue";
import SubmitButton from "@/components/form/SubmitButton.vue";
import TextInput from "@/components/form/text/TextInput.vue";
import TwoByFormBox from "@/components/form/TwoByFormBox.vue";
import type { StorageItem } from "@/components/nr/storage/storageTypes";
import http from "@/http";
import router from "@/router";
import { useRepositoryStore } from "@/stores/repositories";
import { getConfigType, type RepositoryTypeDescription } from "@/types/repository";
import { notify } from "@kyvg/vue3-notification";
import { computed, ref, watch } from "vue";
const input = ref({
  name: "",
  storage: "",
});
const repoTypesStore = useRepositoryStore();
const selectedRepositoryType = ref("");
const repositoryTypes = ref<RepositoryTypeDescription[]>([]);
const storages = ref<StorageItem[]>([]);
const storageItemOptions = computed(() => {
  return storages.value.map((storage) => {
    return {
      value: storage.id,
      label: `${storage.name} (${storage.storage_type})`,
    };
  });
});
const repositoryTypeOptions = computed(() => {
  return repositoryTypes.value.map((type) => {
    return {
      value: type.type_name,
      label: type.name,
    };
  });
});
const currentRepositoryType = computed(() => {
  return repositoryTypes.value.find((type) => type.type_name === selectedRepositoryType.value);
});
const requiredConfigValues = ref<Record<string, any>>({});
watch(selectedRepositoryType, (newValue, old) => {
  if (newValue !== old) {
    console.log(`Changed repository type to ${newValue} from '${old}'. Resetting required configs`);
    requiredConfigValues.value = {} as Record<string, any>;
    for (const config of currentRepositoryType.value?.required_configs || []) {
      requiredConfigValues.value[config] = {} as any;
    }
  }
});
watch(requiredConfigValues, () => {
  console.log(requiredConfigValues.value);
});
const requiredConfigComponents = computed(() => {
  if (!currentRepositoryType.value) {
    return [];
  }

  const configs = currentRepositoryType.value?.required_configs.map((config) => {
    const component = getConfigType(config);
    if (component) {
      return {
        component: component.component,
        configName: config,
      };
    } else {
      return {
        component: FallBackEditor,
        configName: config,
        props: {
          settingName: config,
        },
      };
    }
  });
  console.log(configs);
  return configs;
});

async function load() {
  await repoTypesStore.getStorages(true).then((response) => {
    storages.value = response;
  });

  await repoTypesStore.getRepositoryTypes().then((response) => {
    repositoryTypes.value = response;
  });
}

load();

async function createRepository() {
  const request = {
    name: input.value.name,
    storage: input.value.storage,
    configs: {} as any,
  };
  for (const [key, value] of Object.entries(requiredConfigValues.value)) {
    console.log(`${key} = ${JSON.stringify(value)}`);
    request.configs[key] = value;
  }
  console.log(JSON.stringify(request));
  await http
    .post(`/api/repository/new/${selectedRepositoryType.value}`, request)
    .then((response) => {
      notify({
        type: "success",
        title: "Success",
        text: "Repository created",
      });
      router.push({
        name: "AdminViewRepository",
        params: { id: response.data.id },
      });
    })
    .catch((error) => {
      notify({
        type: "error",
        title: "Error",
        text: "Failed to create repository",
      });
      console.error(error);
    });
}
</script>
<style scoped lang="scss">
@import "@/assets/styles/theme.scss";
form {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  width: 50%;
  padding: 1rem;
  margin: 0 auto;
}
.storageConfig {
  padding: 1rem;
  border: 1px solid $secondary;
  border-radius: 0.5rem;
}
@media screen and (max-width: 1200px) {
  form {
    width: 100%;
  }
}
main {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}
</style>
