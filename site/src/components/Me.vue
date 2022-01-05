<template></template>

<script lang="ts">
import {User} from "@/backend/Response";
import {defineComponent, ref} from "vue";
import {useCookie} from "vue-cookie-next";
import {getUser} from "@/backend/api/User";
import {updateMyPassword} from "@/backend/api/backend/User";
import {ANON_USER} from "@/store/user";

export default defineComponent({
  setup() {
    let password = ref({
      password: "",
      confirm: "",
      error: "",
    });

    const isLoading = ref(false);
    const cookie = useCookie();
    const tab = ref(0);
    const user = ref<User>(ANON_USER);
    const loadUser = async () => {
      isLoading.value = true;
      try {
        let value = await getUser(cookie.getCookie("token"));

        user.value = value as User;

        isLoading.value = false;
      } catch (e) {}
    };
    loadUser();

    return { user, password, tab, isLoading };
  },
  methods: {
    async updatePassword() {
      if (this.password.password != this.password.confirm) {
        this.$notify({
          title: "Passwords do not match",
          type: "error",
        });
      }
      const response = await updateMyPassword(
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
