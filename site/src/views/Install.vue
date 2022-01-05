<template></template>

<script lang="ts">
import {installRequest} from "@/backend/api/Install";
import {defineComponent, ref} from "vue";

export default defineComponent({
  setup() {
    let form = ref({
      email: "",
      name: "",
      username: "",
      password: "",
      confirm_password: "",
    });
    return { form };
  },
  methods: {
    async onSubmit() {
      let install = await installRequest(
        this.form.name,
        this.form.username,
        this.form.password,
        this.form.confirm_password,
        this.form.email
      );
      if (install.ok && install.val) {
        this.$notify({
          title: "Unable to Install Nitro_Repo. Check Logs",
          type: "success",
        });
      } else if (install.err) {
        this.$notify({
          title: install.val.user_friendly_message,
          type: "warn",
        });
      }
    },
  },
});
</script>
<style scoped></style>
