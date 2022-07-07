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
import {
  createNewRepository,
  Repository,
  Storage,
} from "@nitro_repo/nitro_repo-api-wrapper";
import { defineComponent, inject, ref, watch } from "vue";
import { useRouter } from "vue-router";

export default defineComponent({
  props: {
    storage: {
      type: Object as () => Storage,
      required: true,
    },
    modelValue: Boolean,
  },
  setup(props, { emit }) {
    const token: string | undefined = inject("token");
    if (token == undefined) {
      useRouter().push("login");
    }
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
      token: token as string,
    };
  },
  methods: {
    async onSubmit() {
      if (this.form.type === "") {
        this.$notify({
          title: "Please Specify a Repository Type",
          type: "warn",
        });
        return;
      }
      const response = await createNewRepository(
        this.form.name,
        this.$props.storage.name,
        this.form.type,
        this.token
      );
      if (response.ok) {
        const data = response.val as Repository;
        this.$notify({
          title: "Repository Created",
          type: "success",
        });
        this.$router.push(
          "/admin/repository/" + data.storage + "/" + data.name
        );
      } else {
        this.$notify({
          title: "Unable to Create Repository",
          text: JSON.stringify(response.val.user_friendly_message),
          type: "error",
        });
      }
    },
  },
});
</script>
<style scoped></style>
