<template>
  <div class="settingContent">
    <h2 class="settingHeader">Repository Rules</h2>

    <div class="flex flex-wrap mb-6 justify-center">
      <div class="settingBox">
        <label class="nitroLabel" for="grid-name"> name </label>
        <input
          class="disabled nitroTextInput"
          id="grid-name"
          type="text"
          v-model="repository.name"
          disabled
        />
      </div>
      <div class="settingBox">
        <label class="nitroLabel" for="grid-Storage"> Storage </label>
        <input
          class="disabled nitroTextInput"
          id="grid-Storage"
          type="text"
          v-model="repository.storage"
          disabled
        />
      </div>
      <div class="settingBox">
        <label class="nitroLabel" for="grid-created"> Date Created </label>
        <input
          class="disabled nitroTextInput"
          id="grid-created"
          type="text"
          v-model="date"
          disabled
        />
      </div>
      <div class="settingBox">
        <label class="nitroLabel" for="grid-type"> Repo Type</label>
        <input
          class="disabled nitroTextInput"
          id="grid-type"
          type="text"
          v-model="repositoryType"
          disabled
        />
      </div>
    </div>
    <h2 class="settingHeader">Repository General Properties</h2>
    <div class="flex flex-wrap mb-6">
      <div class="settingBox">
        <label class="nitroLabel" for="grid-policy"> Repo Policy</label>
        <select
          v-model="repository.settings.policy"
          class="nitroSelectBox"
          @change="updatePolicy()"
        >
          <option>Mixed</option>
          <option>Release</option>
          <option>Snapshot</option>
        </select>
      </div>

      <div class="settingBox">
        <label class="nitroLabel" for="grid-active">Repo Active</label>
        <select
          v-model="repository.settings.active"
          class="nitroSelectBox"
          @change="updateActiveStatus()"
        >
          <option>true</option>
          <option>false</option>
        </select>
      </div>
    </div>
    <h2 class="settingHeader">Danger Area</h2>
    <div class="settingContent">
      <div class="settingBox">
        <button @click="deleteOpen = true" class="nitroButton">
          Delete Repository
        </button>
      </div>
    </div>
  </div>
  <NitroModal v-model="deleteOpen">
    <template v-slot:header>
      Delete {{ repository.storage }}/{{ repository.name }}
    </template>
    <template v-slot:content>
      <form class="flex flex-col w-96 <sm:w-65" @submit.prevent="deleteRepo()">
        <div class="mb-4">
          <Switch id="deleteFiles" v-model="deleteFiles">
            <div class="ml-3 text-slate-50 font-medium">Delete Files</div>
          </Switch>
        </div>
        <button
          class="bg-slate-900 py-2 my-3 hover:bg-red-700 rounded-md cursor-pointer text-white"
        >
          Delete Repository
        </button>
      </form>
    </template>
  </NitroModal>
</template>
<script lang="ts">
import { defineComponent, inject, ref } from "vue";
import { Repository } from "@/types/repositoryTypes";

export default defineComponent({
  props: {
    repository: {
      required: true,
      type: Object as () => Repository,
    },
  },
  data(props) {
    const token = inject("token") as string;
    console.log(props.repository.repo_type);
    return { token };
  },
  setup(props) {
    const deleteOpen = ref(false);
    const deleteFiles = ref(false);
    const repositoryType = Object.keys(props.repository.repo_type)[0];
    const date = new Date(props.repository.created).toLocaleDateString("en-US");
    return { repositoryType, date, deleteOpen, deleteFiles };
  },
  methods: {
    async updateActiveStatus() {
      // TODO update active status
    },

    async deleteRepo() {
      // TODO delete repo
    },
    async updatePolicy() {
      // TODO update policy
    },
  },
  components: {},
});
</script>
