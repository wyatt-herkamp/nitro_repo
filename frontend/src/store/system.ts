import { acceptHMRUpdate, defineStore } from "pinia";
import httpCommon from "@/http-common";
import { useCookies } from "vue3-cookies";
import { User } from "@/types/userTypes";
import { Version } from "@/types/system";

export const useSystemStore = defineStore({
  id: "system",
  state: () => ({
    version: <Version | undefined>undefined,
  }),

  actions: {
    async load() {
      await httpCommon.apiClient.get<Version>("api/version").then((result) => {
        console.log(result.data);
        this.$patch({
          version: result.data,
        });
      });
    },
  },
});

if (import.meta.hot) {
  import.meta.hot.accept(acceptHMRUpdate(useSystemStore, import.meta.hot));
}
