<template></template>

<script lang="ts">
import {User} from "@/backend/Response";
import {createNewUser} from "@/backend/api/admin/User";
import {defineComponent, ref} from "vue";

export default defineComponent({
  props: {
    updateList: {
      required: true,
      type: Function,
    },
  },
  setup() {
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
    return { form };
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
        this.$props.updateList(data.id);
        this.$notify({
          title: "User Created",
          type: "success",
        });
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
<style scoped></style>
