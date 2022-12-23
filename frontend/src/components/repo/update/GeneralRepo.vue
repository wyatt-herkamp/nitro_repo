<template>
  <div class="settingContent">
    <h2 class="settingHeader">Repository Rules</h2>

    <div class="flex flex-wrap mb-6 justify-center bg-tertiary/25 shadow-sm">
      <table class="table-auto text-quaternary">
        <tbody>
          <tr>
            <th scope="row">Name</th>
            <td>{{ repository.name }}</td>
          </tr>
          <tr>
            <th scope="row">Created</th>
            <td>{{ date }}</td>
          </tr>
          <tr>
            <th scope="row">Type</th>
            <td>{{ repositoryType }}</td>
          </tr>
          <tr>
            <th scope="row">Storage</th>
            <td>{{ repository.storage }}</td>
          </tr>
          <tr>
            <th scope="row">Active</th>
            <td>
              <select
                :value="repository.active"
                class="tableSelectBox"
                @change="updateActiveStatus($event)"
              >
                <option>true</option>
                <option>false</option>
              </select>
            </td>
          </tr>
          <tr>
            <th scope="row">Require Auth Token</th>
            <td>
              <select
                :value="repository.require_token_over_basic"
                class="tableSelectBox"
                @change="updateRequireAuth($event)"
              >
                <option>true</option>
                <option>false</option>
              </select>
            </td>
          </tr>
          <tr>
            <th scope="row">Visibility</th>
            <td>
              <select
                :value="repository.visibility"
                class="tableSelectBox"
                @change="updateVisibility($event)"
              >
                <option value="Public">Public</option>
                <option value="Private">Private</option>
                <option value="Hidden">Hidden</option>
              </select>
            </td>
          </tr>
        </tbody>
      </table>
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
import httpCommon from "@/http-common";
import NitroModal from "@/components/common/model/NitroModal.vue";
import Switch from "@/components/common/forms/Switch.vue";

export default defineComponent({
  props: {
    repository: {
      required: true,
      type: Object as () => Repository,
    },
  },
  setup(props) {
    const deleteOpen = ref(false);
    const deleteFiles = ref(false);
    const repositoryType = ref(props.repository.repository_type);
    const date = new Date(props.repository.created).toLocaleDateString("en-US");
    return { repositoryType, date, deleteOpen, deleteFiles };
  },
  methods: {
    async updateActiveStatus(event: { target: { value: string } }) {
      await httpCommon.apiClient
        .put(
          `api/admin/repositories/${this.repository.storage}/${this.repository.name}/config/active/${event.target.value}`
        )
        .then((response) => {
          if (response.status === 204) {
            this.$notify({
              type: "success",
              title: "Repository active status updated",
            });
          } else {
            this.$notify({
              type: "error",
              title: "Error",
              text: "Repository active status update failed",
            });
          }
        })
        .catch((error) => {
          console.log(error);
          this.$notify({
            type: "error",
            title: "Error",
            text: "Repository active status update failed",
          });
        });
    },
    async updateRequireAuth(event: { target: { value: string } }) {
      await httpCommon.apiClient
        .put(
          `api/admin/repositories/${this.repository.storage}/${this.repository.name}/config/require_token_over_basic/${event.target.value}`
        )
        .then((response) => {
          if (response.status === 204) {
            this.$notify({
              type: "success",
              title: "Repository Require Auth Token updated",
            });
          } else {
            this.$notify({
              type: "error",
              title: "Error",
              text: "Repository Require Auth Token failed",
            });
          }
        })
        .catch((error) => {
          console.log(error);
          this.$notify({
            type: "error",
            title: "Error",
            text: "Repository Require Auth Token failed",
          });
        });
    },
    async updateVisibility(event: { target: { value: string } }) {
      await httpCommon.apiClient
        .put(
          `api/admin/repositories/${this.repository.storage}/${this.repository.name}/config/visibility/${event.target.value}`
        )
        .then((response) => {
          if (response.status === 204) {
            this.$notify({
              type: "success",
              title: "Repository visibility updated",
            });
          } else {
            console.log(response);
            this.$notify({
              type: "error",
              title: "Repository visibility update failed",
            });
          }
        })
        .catch((error) => {
          console.log(error);
          this.$notify({
            type: "error",
            title: "Repository visibility update failed",
          });
        });
    },

    async deleteRepo() {
      // TODO delete repo
    },
  },
  components: { Switch, NitroModal },
});
</script>
<style>
.tableSelectBox {
  @apply block;
  @apply w-full;
  @apply bg-tertiary;
  @apply text-quaternary;
  @apply border;
  @apply border-quaternary;
  @apply py-1;
  @apply px-1;
  @apply w-1/2;
}
</style>
