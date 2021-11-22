<template>
  <el-menu :default-active="activeIndex" class="el-menu-demo" mode="horizontal">
    <el-menu-item index="Home" @click="router.push('/')">Index</el-menu-item>
    <el-menu-item index="Browse" @click="router.push('/browse')"
      >Browse</el-menu-item
    >
    <el-menu-item
        v-if="user.id != 0"
        index="Admin"
        @click="router.push('/admin')"
    >Admin
    </el-menu-item
    >
    <el-menu-item v-else index="Login" @click="dialogVisible = true"
    >Login
    </el-menu-item
    >
  </el-menu>
  <el-dialog v-model="dialogVisible" title="Login" width="30%">
    <el-form :model="form" label-position="top" label-width="120px" v-on:submit="onSubmit">
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
      <el-form-item>
        <el-button block native-type="submit" type="primary"
        >Log In
        </el-button
        >
      </el-form-item>
    </el-form>
  </el-dialog>
</template>

<script lang="ts">
import {defineComponent, ref} from "vue";
import {useRouter} from "vue-router";
import {AuthToken, BasicResponse, User} from "@/backend/Response";
import http from "@/http-common";

export default defineComponent({
  props: {
    user: {
      required: true,
      type: Object as () => User,
    },
  },
  setup() {
    let form = ref({
      username: "",
      password: "",
    });
    const router = useRouter();
    const activeIndex = ref(router.currentRoute.value.name);
    const dialogVisible = ref(false);
    return {activeIndex, router, dialogVisible, form};
  },
  methods: {
    async onSubmit(e: any) {
      e.preventDefault()
      let newUser = {
        username: this.form.username,
        password: this.form.password,
      };
      let body = JSON.stringify(newUser);
      console.log(body);
      const res = await http
          .post("api/login", body)
          .then((res) => {
            console.log(typeof res);
            if (res.status != 200) {
              return;
            }
            const result = res.data;
            let value = JSON.stringify(result);

            let response: BasicResponse<unknown> = JSON.parse(value);

            if (response.success) {
              let loginRequest = response as BasicResponse<AuthToken>;
              let date = new Date(loginRequest.data.expiration * 1000);
              this.$cookie.setCookie("token", loginRequest.data.token, {
                expire: date,
                sameSite: "lax",
              });
              this.dialogVisible = false;
              location.reload();
            } else {
              this.form.password = "";
              this.$notify({
                title: "Invalid Username or Password",
                type: "warn",
              });
            }
          })
          .catch((error) => {
            if (error.response) {
              if (error.response.status == 401) {
                this.form.password = "";
                this.$notify({
                  title: "Invalid Username or Password",
                  type: "warn",
                });
              } else {
                this.$notify({
                  title: "Unkown Error Occured",
                  type: "warn",
                });
              }
            }
          });
    },
  },
});
</script>
