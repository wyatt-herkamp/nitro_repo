<template>
  <div class="md:w-1/4 mx-auto">
    <div class="py-2">
      <h1 class="text-quaternary text-3xl py-2">Welcome</h1>
      <h6 class="text-sm text-quaternary/50">
        Login to your Nitro Repo Account
      </h6>
    </div>
    <LoginComp @login="handleLogin" />
  </div>
</template>

<script lang="ts">
import { defineComponent } from "vue";
import LoginComp from "@/components/user/LoginComp.vue";
export default defineComponent({
  components: { LoginComp },
  methods: {
    handleLogin(status: string) {
      console.log(status);
      if (status === "success") {
        this.$route.query.redirect
          ? this.$router.resolve(this.$route.query.redirect as string)
          : this.$router.resolve("/");
      } else if (status === "failure") {
        this.$notify({
          title: "Login Failed",
          type: "warn",
        });
      } else {
        this.$notify({
          title: "Login Failed",
          type: "error",
        });
      }
    },
  },
});
</script>
