
<template>
  <NitroModal v-model="showModel">
    <template v-slot:header> Create User </template>
    <template v-slot:content>
      <form class="flex flex-col w-96 sm:w-65" @submit.prevent="onSubmit()">
        <div class="flex flex-row">
          <div class="settingBox">
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
          <div class="settingBox">
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
        <div class="flex flex-row">
          <div class="px-3 w-96 sm:w-65">
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
        <div class="flex flex-row">
          <div class="settingBox">
            <label class="nitroLabel"> Password </label>
            <input
              class="nitroTextInput"
              id="nitroLabel"
              type="password"
              v-model="form.password.password"
              required
            />
          </div>
          <div class="settingBox">
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
        <button class="nitroButtonLight">Create User</button>
      </form>
    </template>
    <template v-slot:button>
      <button class="openModalButton">Create User</button>
    </template>
  </NitroModal>
</template>
<script lang="ts">
import { User } from "nitro_repo-api-wrapper";
import { createNewUser } from "nitro_repo-api-wrapper";
import { defineComponent, ref } from "vue";

export default defineComponent({
  setup() {
    const showModel = ref(false);
    const close = () => (showModel.value = false);

    let form = ref({
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
    return { form, showModel, close };
  },
  methods: {
    async onSubmit() {
      if (this.form.password.password != this.form.password.password_two) {
        this.$notify({
          title: "Passwords do not match",
          type: "error",
        });
      }
      const response = await createNewUser(
        this.form.name,
        this.form.username,
        this.form.password.password,
        this.form.email,
        this.$cookie.getCookie("token")
      );
      if (response.ok) {
        let data = response.val as User;
        this.$notify({
          title: "User Created",
          type: "success",
        });
        this.$router.push("/admin/user/" + data.id);
      } else {
        this.$notify({
          title: "Unable to Create user",
          text: JSON.stringify(response.val.user_friendly_message),
          type: "error",
        });
      }
    },
  },
});
</script>
<style scoped>
.settingBox {
  @apply md:w-1/2;
  @apply px-3;
}
</style>
