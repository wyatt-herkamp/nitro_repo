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
          <label class="nitroLabel" for="name"> Storage ID/Name </label>
          <input
            id="name"
            v-model="form.name"
            autocomplete="off"
            class="nitroTextInput"
            placeholder="Storage ID/Name"
            type="text"
          />
        </div>
      </div>
      <div class="flex-row">
        <div class="px-3">
          <label class="nitroLabel" for="name"> Storage Public Name </label>
          <input
            id="name"
            v-model="form.public_name"
            autocomplete="off"
            class="nitroTextInput"
            placeholder="Public Name"
            type="text"
          />
        </div>
      </div>
      <div class="flex flex-row h-12 mt-5">
        <button class="buttonOne">Create User</button>
      </div>
    </form>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref, watch } from "vue";
import "@/styles/sideCreate.css";
import httpCommon from "@/http-common";
export default defineComponent({
  props: {
    modelValue: Boolean,
  },
  setup(props, { emit }) {
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
    const form = ref({
      name: "",
      public_name: "",
      error: "",
    });
    return { form, showModel, close };
  },
  methods: {
    async onSubmit() {
      await httpCommon.apiClient
        .post("api/admin/storage/new", {
          storage_type: "LocalStorage",
          name: this.form.name,
          public_name: this.form.public_name,
          handler_config: {
            location: `./${this.form.name}`,
          },
        })
        .then((res) => {
          if (res.status == 200) {
            this.$notify({
              title: "Created Storage",
              type: "success",
            });
          } else if (res.status == 409) {
            this.$notify({
              title: "Storage Already Exists",
              type: "error",
            });
          } else {
            this.$notify({
              title: "Error Creating Storage",
              text: res.data,
              type: "error",
            });
          }
        });
    },
  },
});
</script>
<style scoped></style>
