<template>
  <el-container direction="horizontal" style="border: 1px solid #eee">
    <el-main>
      <el-alert
        v-if="form.error.length != 0"
        :title="form.error"
        type="error"
      />
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
            v-model="this.form.password.password"
            placeholder="Please input password"
            show-password
          />
        </el-form-item>
        <el-form-item label="Confirm Password">
          <el-input
            v-model="form.password.password_two"
            placeholder="Please input password"
            show-password
          />
        </el-form-item>
        <el-form-item>
          <el-button type="primary" @click="onSubmit">Login</el-button>
        </el-form-item>
      </el-form>
    </el-main>
  </el-container>
</template>

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
