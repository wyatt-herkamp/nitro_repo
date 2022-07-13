<template>
  <div class="sideCreate">
    <form @submit.prevent="onSubmit()">
      <div class="sideHeader">
        <p class="headerOne">Create User</p>
        <button type="button" class="xButton" @click="showModel = false">
          🗙
        </button>
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
            <option value="Maven">Maven</option>
            <option value="NPM">NPM</option>
          </select>
        </div>
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

export default defineComponent({
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
    const showModel = ref(props.modelValue);
    watch(
      () => props.modelValue,
      (val) => {
        showModel.value = val;
        emit("update:modelValue", val);
      }
    );
    watch(showModel, (val) => {
      emit("update:modelValue", val);
    });
    return {
      form,
      showModel,
    };
  },
  methods: {
    async onSubmit() {
      const storageName = this.storage.name;
      await httpCommon.apiClient
        .post(
          `/api/admin/repositories/${storageName}/new/${this.form.name}/${this.form.type}`,
          {}
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
