<template>
  <div id="login-modal">
    <vue-final-modal
      v-model="showLogin"
      classes="flex justify-center items-center"
    >
      <div
        class="relative border bg-slate-900 border-black m-w-20 py-5 px-10 rounded-2xl shadow-xl text-center"
      >
        <p class="font-bold text-xl pb-4">Login</p>
        <form
          class="flex flex-col w-96 sm:w-65"
          @submit.prevent="onSubmit(form.username, form.password)"
        >
          <input
            id="username"
            v-model="form.username"
            autocomplete="username"
            class="input"
            placeholder="Username"
            type="text"
          />

          <input
            id="password"
            v-model="form.password"
            autocomplete="current-password"
            class="input"
            placeholder="Password"
            type="password"
          />
          <button
            class="bg-slate-800 py-2 my-3 rounded-md cursor-pointer text-white"
          >
            Sign in
          </button>
        </form>

        <button class="absolute top-0 right-0 mt-5 mr-5" @click="close()">
          🗙
        </button>
      </div>
    </vue-final-modal>
    <div @click="showLogin = true">
      <slot name="button"></slot>
    </div>
  </div>
</template>

<script lang="ts">
import {defineComponent, ref} from "vue";
import {useRouter} from "vue-router";
import {login} from "@/backend/api/backend/User";
import {AuthToken} from "@/backend/api/User";

export default defineComponent({
  setup() {
    const showLogin = ref(false);

    const close = () => (showLogin.value = false);
    let form = ref({
      username: "",
      password: "",
    });
    const router = useRouter();
    const activeIndex = ref(router.currentRoute.value.name);
    const dialogVisible = ref(false);
    return { activeIndex, router, dialogVisible, form, showLogin, close };
  },
  methods: {
    async onSubmit(username: string, password: string) {
      const value = await login(username, password);
      if (value.ok) {
        let loginRequest = value.val as AuthToken;
        let date = new Date(loginRequest.expiration * 1000);
        this.$cookie.setCookie("token", loginRequest.token, {
          expire: date,
          sameSite: "lax",
        });
        this.dialogVisible = false;
        location.reload();
      } else {
        this.form.password = "";
        this.$notify({
          title: value.val.user_friendly_message,
          type: "warn",
        });
      }
    },
  },
});
</script>
<style scoped>
.input {
  @apply p-2;
  @apply my-1;
  @apply bg-slate-50 dark:bg-slate-800;
  @apply rounded-md;
  @apply text-white;
}

#login-modal button:hover {
  @apply bg-slate-200 dark:bg-slate-700;
  transition: background-color 0.5s;
}
</style>
