<template></template>

<script lang="ts">
import {User, UserListResponse} from "@/backend/Response";
import {defineComponent, ref} from "vue";
import {useCookie} from "vue-cookie-next";
import {getUserByID} from "@/backend/api/User";
import {updateNameAndEmail, updateOtherPassword, updatePermission,} from "@/backend/api/admin/User";
import {ANON_USER} from "@/store/user";

export default defineComponent({
  props: {
    userResponse: {
      required: false,
      type: Object as () => UserListResponse,
    },
  },

  setup(props) {
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
    const user = ref<User>(ANON_USER);
    const loadUser = async () => {
      isLoading.value = true;
      try {
        let value = (await getUserByID(
          cookie.getCookie("token"),
          (props.userResponse as UserListResponse).id
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

    return { user, settingForm, password, tab, isLoading };
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
});
</script>
<style scoped></style>
