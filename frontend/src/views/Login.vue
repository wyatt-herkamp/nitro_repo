<template>
  <div class="md:w-1/4 mx-auto">
    <div class="py-2 text-left font-bold text-lg">
      <h1>Login</h1>
    </div>
    <LoginComp  />
  </div>
</template>

<script lang="ts">
import { defineComponent } from "vue";
import LoginComp from "@/components/user/LoginComp.vue";
export default defineComponent({
  components: { LoginComp },
});
</script>
