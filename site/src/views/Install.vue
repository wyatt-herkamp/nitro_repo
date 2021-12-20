<template>
  <el-container direction="horizontal" style="border: 1px solid #eee">
    <el-main>
      <el-form label-position="top" :model="form" label-width="120px">
        <el-form-item label="Email">
          <el-input v-model="form.email" autocomplete="email"></el-input>
        </el-form-item>
        <el-form-item label="Name">
          <el-input v-model="form.name"></el-input>
        </el-form-item>
        <el-form-item label="Username">
          <el-input v-model="form.username"></el-input>
        </el-form-item>
        <el-form-item label="Password">
          <el-input
            v-model="this.form.password"
            placeholder="Please input password"
            show-password
          />
        </el-form-item>
        <el-form-item label="Confirm Password">
          <el-input
            v-model="form.confirm_password"
            placeholder="Please input password"
            show-password
          />
        </el-form-item>
        <el-form-item>
          <el-button type="primary" @click="onSubmit">Install</el-button>
        </el-form-item>
      </el-form>
    </el-main>
  </el-container>
</template>

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
