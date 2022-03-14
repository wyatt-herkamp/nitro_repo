<template>
  <div
    v-if="user != undefined"
    class="min-h-screen w-full flex flex-wrap lg:flex-nowrap"
  >
    <div class="flex flex-col w-full">
      <UserEditMenu @changeView="view = $event" />
      <div class="flex flex-col float-right w-auto">
        <div class="settingContent" v-if="view == 'General'">
          <h2 class="text-white m-3 text-left">User</h2>

          <div class="flex flex-wrap mb-6 justify-center">
            <div class="settingBox">
              <label for="grid-name"> name </label>
              <input
                class="disabled"
                id="grid-name"
                type="text"
                v-model="user.name"
                disabled
              />
            </div>
            <div class="settingBox">
              <label for="grid-name"> Username </label>
              <input
                class="disabled"
                id="grid-name"
                type="text"
                v-model="user.username"
                disabled
              />
            </div>
            <div class="settingBox">
              <label for="grid-name"> Email </label>
              <input
                class="disabled"
                id="grid-name"
                type="text"
                v-model="user.email"
                disabled
              />
            </div>
          </div>
        </div>
        <div class="settingContent" v-if="view == 'Password'">
          <h2 class="text-white m-3 text-left">Password</h2>
        </div>
        <div class="settingContent" v-if="view == 'Permissions'">
          <h2 class="text-white m-3 text-left">Permissions</h2>

          <div class="flex flex-wrap mb-6 justify-center">
            <div class="settingBox"></div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import { User, UserListResponse } from "@/backend/Response";
import { defineComponent, ref } from "vue";
import { useCookie } from "vue-cookie-next";
import { getUserByID } from "@/backend/api/User";
import {
  updateNameAndEmail,
  updateOtherPassword,
  updatePermission,
} from "@/backend/api/admin/User";
import UserEditMenu from "./user/edit/UserEditMenu.vue";

export default defineComponent({
  props: {
    userID: {
      required: true,
      type: Number,
    },
  },
  setup(props) {
    let view = ref("General");
    let settingForm = ref({
      email: "",
      name: "",
      error: "",
      success: "",
    });
    let password = ref({
      password: "",
      confirm: "",
      error: "",
    });
    const isLoading = ref(false);
    const error = ref("");
    const cookie = useCookie();
    const tab = ref(0);
    const user = ref<User | undefined>();
    const loadUser = async () => {
      isLoading.value = true;
      try {
        let value = (await getUserByID(
          cookie.getCookie("token"),
          props.userID
        )) as User;
        user.value = value as User;
        isLoading.value = false;
        settingForm.value = {
          email: user.value.email,
          name: user.value.name,
          error: "",
          success: "",
        };
        password.value = {
          password: "",
          confirm: "",
          error: "",
        };
      } catch (e) {
        error.value = "";
      }
    };
    loadUser();
    return { user, settingForm, password, tab, isLoading, view };
  },
  methods: {
    settingButton() {
      if (this.user == undefined) return true;
      let user = this.user as User;
      return (
        user.name == this.settingForm.name &&
        user.email == this.settingForm.email
      );
    },
    async onSettingSubmit() {
      if (this.user == undefined) {
        this.$notify({
          title: "Unable Update Name and Email",
          text: "User is still undefined",
          type: "error",
        });
        return;
      }
      const response = await updateNameAndEmail(
        this.user.username,
        this.settingForm.name,
        this.settingForm.email,
        this.$cookie.getCookie("token")
      );
      if (response.ok) {
        let data = response.val as User;
        this.$notify({
          title: "User Updated",
          type: "success",
        });
        this.user = data;
      } else {
        this.$notify({
          title: "Unable Update User",
          text: JSON.stringify(response.val.user_friendly_message),
          type: "error",
        });
      }
    },
    async onPermissionUpdate(permission: string) {
      if (this.user == undefined) {
        this.$notify({
          title: "Unable Update Permission",
          text: "User is still undefined",
          type: "error",
        });
        return;
      }
      let user = this.user as User;
      let value: boolean = user.permissions[permission];
      const response = await updatePermission(
        this.user.username,
        permission,
        value,
        this.$cookie.getCookie("token")
      );
      if (response.ok) {
        this.$notify({
          title: "Updated Permission: " + permission + ": " + value,
          type: "info",
        });
      } else {
        this.$notify({
          title: "Unable Update Password",
          text: JSON.stringify(response.val.user_friendly_message),
          type: "error",
        });
      }
    },
    async updatePassword() {
      if (this.user == undefined) {
        this.$notify({
          title: "Unable Update Password",
          text: "User is still undefined",
          type: "error",
        });
        return;
      }
      if (this.password.password != this.password.confirm) {
        this.$notify({
          title: "Passwords do not match",
          type: "error",
        });
      }
      const response = await updateOtherPassword(
        this.user.username,
        this.password.password,
        this.$cookie.getCookie("token")
      );
      if (response.ok) {
        let data = response.val as User;
        this.$notify({
          title: "Password Updated",
          type: "success",
        });
      } else {
        this.$notify({
          title: "Unable Update Password",
          text: JSON.stringify(response.val.user_friendly_message),
          type: "error",
        });
      }
    },
  },
  components: { UserEditMenu },
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
.toggle-bg:after {
  content: "";
  @apply absolute top-0.5 left-0.5 bg-white border border-gray-300 rounded-full h-5 w-5 transition shadow-sm;
}

input:checked + .toggle-bg:after {
  transform: translateX(100%);
  @apply border-white;
}
.settingContent {
  @apply max-w-lg;
  @apply mx-auto;
}
input:checked + .toggle-bg {
  @apply bg-blue-600 border-blue-600;
}
</style>