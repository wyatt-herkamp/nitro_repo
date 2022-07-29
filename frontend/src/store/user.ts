import { acceptHMRUpdate, defineStore } from "pinia";
import { User } from "@/types/user";
import httpCommon from "@/http-common";
import { useCookies } from "vue3-cookies";

export const useUserStore = defineStore({
  id: "user",
  state: () => ({
    user: <User | undefined>undefined,
    date: new Date(),
  }),

  actions: {
    logout() {
      this.$patch({ user: undefined, date: new Date() });
    },

    async loadUser() {
      await httpCommon.apiClient.get<User>("api/me").then((result) => {
        if (result.status == 200) {
          this.$patch({
            user: result.data,
            date: new Date(result.data.created).toLocaleDateString("en-US"),
          });
          const cookies = useCookies();
          // This only exists for quick route guard checking. This does not hurt security because the backend will deny any unauthorized access.
          if (cookies.cookies.get("logged_in") !== "true") {
            cookies.cookies.set("logged_in", "true");
          }
        }
      });
    },
  },
});

if (import.meta.hot) {
  import.meta.hot.accept(acceptHMRUpdate(useUserStore, import.meta.hot));
}
