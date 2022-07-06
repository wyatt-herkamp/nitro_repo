<template>
  <div class="flex w-full">
    <form class="appearance-none flex-col" @submit.prevent="onSubmit()">
      <div class="flex flex-row">
        <div class="grow">
          <p class="header">Create User</p>
        </div>
        <div class="m-auto pt-5 pr-3">
          <button
            type="button"
            class="xButton block"
            @click="showModel = false"
          >
            🗙
          </button>
        </div>
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
      <button class="nitroButtonLight">Create User</button>
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
        this.token as string
      );
      if (response.ok) {
        const data = response.val as User;
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
.xButton {
  @apply float-right;
}
.header {
  @apply font-bold;
  @apply text-xl;
  @apply pb-2;
  @apply pt-2;
  @apply text-left;
  @apply w-3/4;
}
</style>
