<template>
  <div>
    <vue-final-modal
      v-model="showModel"
      classes="flex justify-center items-center"
    >
      <div
        class="
          relative
          border
          bg-slate-900
          border-black
          m-w-20
          py-5
          px-10
          rounded-2xl
          shadow-xl
          text-center
        "
      >
        <p class="font-bold text-xl pb-4">Create User</p>
        <form class="flex flex-col w-96 sm:w-65" @submit.prevent="onSubmit()">
          <div class="flex flex-row">
            <div class="settingBox">
              <label for="grid-name"> Name </label>
              <input
                class="text-input"
                id="grid-name"
                type="text"
                v-model="form.name"
              />
            </div>
            <div class="settingBox">
              <label for="grid-name"> Username </label>
              <input
                class="text-input"
                id="grid-name"
                type="text"
                v-model="form.username"
              />
            </div>
          </div>
          <div class="flex flex-row flex-grow my-2">
            <div class="settingBox">
              <label for="grid-name"> Email </label>
              <input
                class="email"
                id="grid-name"
                type="email"
                v-model="form.email"
              />
            </div>
          </div>
          <div class="flex flex-row my-2">
            <div class="settingBox">
              <label for="grid-name"> Password </label>
              <input
                class="text-input"
                id="grid-name"
                type="password"
                v-model="form.password.password"
              />
            </div>
            <div class="settingBox">
              <label for="grid-name"> Confirm Password </label>
              <input
                class="text-input"
                id="grid-name"
                type="password"
                v-model="form.password.password_two"
              />
            </div>
          </div>
          <button
            class="bg-slate-800 py-2 my-3 rounded-md cursor-pointer text-white"
          >
            Create User
          </button>
        </form>

        <button class="absolute top-0 right-0 mt-5 mr-5" @click="close()">
          🗙
        </button>
      </div>
    </vue-final-modal>
    <div @click="showModel = true">
      <slot name="button"></slot>
    </div>
  </div>
</template>
<script lang="ts">
import { User } from "@/backend/Response";
import { createNewUser } from "@/backend/api/admin/User";
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
label {
  @apply block;
  @apply uppercase;
  @apply tracking-wide;
  @apply text-white;
  @apply text-xs;
  @apply font-bold;
  @apply text-left;
  @apply my-3;
}
.settingBox {
  @apply md:w-1/2;
  @apply px-3;
}
.disabled {
  @apply appearance-none;
  @apply block;
  @apply w-full;
  @apply bg-gray-300;
  @apply text-gray-700;
  @apply border;
  @apply border-gray-800;
  @apply rounded;
  @apply py-3;
  @apply px-4;
  @apply leading-tight;
}
.text-input {
  @apply appearance-none;
  @apply block;
  @apply w-full;
  @apply bg-gray-200;
  @apply text-gray-700;
  @apply border;
  @apply border-gray-200;
  @apply rounded;
  @apply py-3;
  @apply px-4;
  @apply leading-tight;
  @apply focus:outline-none;
  @apply focus:bg-white;
  @apply focus:border-gray-500;
}
.email {
  @apply appearance-none;
  @apply block;
  @apply bg-gray-200;
  @apply text-gray-700;
  @apply border;
  @apply w-80;
  @apply border-gray-200;
  @apply rounded;
  @apply py-3;
  @apply px-4;
  @apply leading-tight;
  @apply focus:outline-none;
  @apply focus:bg-white;
  @apply focus:border-gray-500;
}
</style>
