<template>
  <form @submit.prevent="onSubmit()">
    <div class="flex flex-row pl-6">
      <p class="headerOne">Create User</p>
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
</template>
<script lang="ts">
import { defineComponent, ref } from "vue";
import httpCommon from "@/http-common";
import { notify } from "@kyvg/vue3-notification";

export default defineComponent({
  setup(props, { emit }) {
    const form = ref({
      name: "",
      username: "",
      email: "",
      password: {
        password: "",
        password_two: "",
      },
    });
    return { form };
  },
  methods: {
    async onSubmit() {
      const { form } = this;
      const { name, username, email, password } = form;
      if (password.password_two !== password.password) {
        this.$notify({
          type: "error",
          title: "Passwords do not match",
        });
        return;
      }
      await httpCommon.apiClient
        .post("/api/admin/user", {
          name,
          username,
          email,
          password: password.password,
        })
        .then((r) => {
          if (r.status === 201) {
            this.$notify({
              type: "success",
              title: "User created",
            });
            this.$emit("close");
          }
        })
        .catch((r) => {
          if (r.response.status === 409) {
            this.$notify({
              type: "warn",
              title: "User already exists",
              text: r.response.data,
            });
          } else {
            this.$notify({
              type: "warn",
              title: "Error",
              text: r.response.data,
            });
          }
        });
    },
  },
});
</script>
