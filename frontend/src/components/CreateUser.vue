<template>
  <div class="sideCreate">
    <form @submit.prevent="onSubmit()">
      <div class="flex flex-row pl-6">
        <p class="headerOne">Create User</p>
        <button type="button" class="xButton" @click="showModel = false">
          🗙
        </button>
      </div>

      <div class="flex-row">
        <div class="px-3">
          <label class="nitroLabel"> Name </label>
          <input
            class="nitroTextInput"
            id="nitroLabel"
            type="text"
            placeholder="Example"
            v-model="form.name"
            required
          />
        </div>
      </div>
      <div class="flex-row">
        <div class="px-3">
          <label class="nitroLabel"> Username </label>
          <input
            class="nitroTextInput"
            id="nitroLabel"
            type="text"
            placeholder="Username"
            v-model="form.username"
            required
          />
        </div>
      </div>
      <div class="flex-row">
        <div class="px-3">
          <label class="nitroLabel"> Email </label>
          <input
            class="nitroTextInput email"
            id="nitroLabel"
            type="email"
            placeholder="example@nitro_repo.kigntux.dev"
            v-model="form.email"
            required
          />
        </div>
      </div>
      <div class="flex flex-row flex-wrap md:flex-nowrap">
        <div class="px-3 md:w-1/2">
          <label class="nitroLabel"> Password </label>
          <input
            class="nitroTextInput"
            id="nitroLabel"
            type="password"
            v-model="form.password.password"
            required
          />
        </div>
        <div class="px-3 md:w-1/2">
          <label class="nitroLabel"> Confirm Password </label>
          <input
            class="nitroTextInput"
            id="nitroLabel"
            type="password"
            v-model="form.password.password_two"
            required
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
import { createNewUser, User } from "@nitro_repo/nitro_repo-api-wrapper";
import { defineComponent, inject, ref, watch } from "vue";
import { useRouter } from "vue-router";

export default defineComponent({
  props: {
    modelValue: Boolean,
  },
  setup(props, { emit }) {
    const token: string | undefined = inject("token");
    if (token == undefined) {
      useRouter().push("login");
    }
    const showModel = ref(props.modelValue);

    watch(
      () => props.modelValue,
      (val) => {
        showModel.value = val;
        emit("update:modelValue", val);
      }
    );
    watch(showModel, (val) => {
      console.log(val);
      emit("update:modelValue", val);
    });

    const form = ref({
      error: "",
      name: "",
      username: "",
      email: "",
      password: {
        password: "",
        password_two: "",
      },
      permissions: { deployer: false, admin: false },
    });
    return { form, showModel, token };
  },
  methods: {
    async onSubmit() {
      //TODO create user
    },
  },
});
</script>
