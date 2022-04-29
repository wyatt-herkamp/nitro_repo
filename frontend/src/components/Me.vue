<template></template>

<script lang="ts">
import { User } from "@nitro_repo/nitro_repo-api-wrapper";
import { defineComponent, inject, ref } from "vue";
import { getUser } from "@nitro_repo/nitro_repo-api-wrapper";
import { updateMyPassword } from "@nitro_repo/nitro_repo-api-wrapper";
import { useRouter } from "vue-router";

export default defineComponent({
  setup() {
    let password = ref({
      password: "",
      confirm: "",
      error: "",
    });

    const isLoading = ref(false);

    const tab = ref(0);
    const user = ref<User | undefined>(undefined);
    const loadUser = async () => {
      isLoading.value = true;
      try {
        let value = await getUser(undefined);

        user.value = value.val as User;

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
        undefined
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
