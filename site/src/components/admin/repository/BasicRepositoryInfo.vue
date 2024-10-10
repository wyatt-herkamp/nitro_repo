<template>
  <div
    v-if="repository"
    id="repository">
    <h2>Repository Info</h2>
    <div id="content">
      <div class="repositoryInfo">
        <div class="twoBy">
          <div class="keyValue">
            <label>Repository Name</label>
            <span class="value">{{ repository.name }}</span>
          </div>
          <div class="keyValue">
            <label>Repository Type</label>
            <span class="value">{{ repository.repository_type }}</span>
          </div>
        </div>
        <div class="twoBy">
          <div class="keyValue">
            <label>Storage Name</label>
            <span class="value">{{ repository.storage_name }}</span>
          </div>
          <div class="keyValue">
            <label>Storage Id Type</label>
            <span class="value">{{ repository.storage_id }}</span>
          </div>
        </div>
      </div>
      <div id="enableDisable">
        <h3>Repository Status {{ repositoryStatus }}</h3>
        <button
          class="disable"
          @click="notify('This feature is not implemented yet')"
          v-if="repository.active">
          Disable
        </button>
        <button
          class="enable"
          @click="notify('This feature is not implemented yet')"
          v-else>
          Enable
        </button>
      </div>
      <div>
        <button
          id="deleteRepository"
          @click="deleteRepository()">
          Delete Repository
        </button>
      </div>
    </div>
  </div>
</template>
<script setup lang="ts">
import http from "@/http";
import router from "@/router";
import type { RepositoryWithStorageName } from "@/types/repository";
import { notify } from "@kyvg/vue3-notification";
import { computed, type PropType } from "vue";
const props = defineProps({
  repository: {
    type: Object as PropType<RepositoryWithStorageName>,
    required: true,
  },
});
const repositoryStatus = computed(() => {
  if (!props.repository) return "No Repository";
  return props.repository.active ? "Active" : "Inactive";
});
async function deleteRepository() {
  http.delete(`/api/repository/${props.repository.id}`).then(() => {
    notify({
      type: "success",
      title: "Deleted",
      text: "Repository Deleted",
    });
    router.push({ name: "RepositoriesList" });
  });
}
</script>
<style lang="scss" scoped>
@import "@/assets/styles/theme.scss";
#repository {
  display: flex;
  flex-direction: column;
}
#content {
  // Make it all the same size spans
  display: flex;
  flex-direction: row;
  gap: 20px;
}
#enableDisable {
  button {
    padding: 10px;
    border-radius: 5px;
    border: none;
    cursor: pointer;
    background-color: $primary;
    &:hover {
      background-color: $primary-50;
      transition: background-color 0.5s;
    }
  }
}
.twoBy {
  display: flex;
  justify-content: space-between;
  gap: 20px;
}
.keyValue {
  display: flex;
  flex-direction: column;
  margin-bottom: 10px;
  width: 100%;
  .value {
    display: block;
    border: 2px solid $secondary-50;
    padding: 5px;
    border-radius: 5px;
    background-color: $secondary;
  }
  // Make it look like a input
  label {
    font-weight: bold;
  }
}
#deleteRepository {
  padding: 10px;
  border-radius: 5px;
  border: none;
  cursor: pointer;
  &:hover {
    transition: background 0.5s;
  }
}
</style>
