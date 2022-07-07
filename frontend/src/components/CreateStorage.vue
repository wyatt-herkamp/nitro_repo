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
import { createNewStorage, Storage } from "@nitro_repo/nitro_repo-api-wrapper";
import { defineComponent, inject, ref, watch } from "vue";
import { useRouter } from "vue-router";
import "@/styles/sideCreate.css";
export default defineComponent({
  props: {
    modelValue: Boolean,
  },
  setup(props, { emit }) {
    const showModel = ref(props.modelValue);
    const token: string | undefined = inject("token");
    if (token == undefined) {
      useRouter().push("login");
    }
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
    return { form, showModel, close, token: token as string };
  },
  methods: {
    async onSubmit() {
      const response = await createNewStorage(
        this.form.name,
        this.form.public_name,
        this.token
      );
      if (response.ok) {
        const data = response.val as Storage;
        this.$notify({
          title: "Storage Created",
          type: "success",
        });
        this.$router.push("/admin/storage/" + data.name);
      } else {
        this.$notify({
          title: "Unable to Create Storage",
          text: JSON.stringify(response.val.user_friendly_message),
          type: "error",
        });
      }
    },
  },
});
</script>
<style scoped></style>
