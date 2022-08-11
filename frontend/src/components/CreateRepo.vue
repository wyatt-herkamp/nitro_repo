<template>
  <div>
    <form @submit.prevent="onSubmit()">
      <div class="sideHeader">
        <p class="headerOne">Create User</p>
      </div>

      <div class="flex-row">
        <div class="px-3">
          <label class="nitroLabel" for="name"> Repository Name </label>
          <input
            id="name"
            v-model="form.name"
            autocomplete="off"
            required
            class="nitroTextInput"
            placeholder="Repository Name"
            type="text"
          />
        </div>
      </div>
      <div class="flex-row">
        <div class="px-3">
          <label class="nitroLabel" for="name"> Repository Type </label>
          <select id="type" v-model="form.type" required class="nitroSelectBox">
            <option disabled selected value="">Repository Type</option>
            <option
              v-for="v in repositoryTypes"
              :value="v.name"
              v-bind:key="v.name"
            >
              {{ v.name }}
            </option>
          </select>
        </div>
      </div>
      <div class="flex-row">
        <MavenCreate v-if="form.type === 'maven'" v-model="innerForm" />
      </div>
      <div class="flex flex-row h-12 mt-5">
        <button class="buttonOne">Create User</button>
      </div>
    </form>
  </div>
</template>
<style scoped></style>
<script lang="ts">
import { defineComponent, ref, watch } from "vue";
import httpCommon from "@/http-common";
import { notify } from "@kyvg/vue3-notification";
import { useRouter } from "vue-router";
import { Storage } from "@/types/storageTypes";
import MavenCreate from "@/components/repo/types/maven/MavenCreate.vue";

export default defineComponent({
  components: { MavenCreate },
  props: {
    storage: {
      type: Object as () => Storage,
      required: true,
    },
    modelValue: Boolean,
  },
  setup(props, { emit }) {
    const form = ref({
      name: "",
      type: "",
      error: "",
    });
    const innerForm = ref<unknown>({});
    const repositoryTypes = ref<Array<{ name: string; layout: unknown }>>();

    httpCommon.apiClient
      .get<Array<{ name: string; layout: unknown }>>(
        "api/admin/tools/repositories/new/layout"
      )
      .then((res) => {
        repositoryTypes.value = res.data;
      });
    return {
      form,
      repositoryTypes,
      innerForm,
    };
  },
  methods: {
    async onSubmit() {
      const storageName = this.storage.id;
      const value = JSON.stringify(this.innerForm);
      console.log(value);
      await httpCommon.apiClient
        .post(
          `/api/admin/repositories/new/${this.form.type}/${storageName}/${this.form.name}`,
          value,
          {
            headers: {
              "Content-Type": "application/json",
            },
          }
        )
        .then((response) => {
          if (response.status == 200) {
            notify({
              title: "Success",
              type: "success",
            });

            useRouter().push({
              name: "AdminRepoView",
              params: {
                storage: storageName,
                repo: this.form.name,
              },
            });
          } else if (response.status == 409) {
            this.$notify({
              title: "Repository already exists",
              type: "error",
            });
          }
        });
    },
  },
});
</script>
<style scoped></style>
