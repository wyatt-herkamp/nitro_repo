<template>

</template>

<script lang="ts">
import {defineComponent, ref} from "vue";
import {useRouter} from "vue-router";
import {User} from "@/backend/Response";
import {login} from "@/backend/api/backend/User";
import {AuthToken} from "@/backend/api/User";

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
    return { activeIndex, router, dialogVisible, form };
  },
  methods: {
    async onSubmit(e: any) {
      e.preventDefault();
      const value = await login(this.form.username, this.form.password);
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
