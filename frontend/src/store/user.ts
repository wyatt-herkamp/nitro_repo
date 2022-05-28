import { getUser, User } from '@nitro_repo/nitro_repo-api-wrapper';
import { defineStore, acceptHMRUpdate } from 'pinia'
import { inject } from 'vue';
import apiClient from '@/http-common';
import { Result } from 'ts-results';
export const useUserStore = defineStore({
  id: 'user',
  state: () => ({
    user: <object | undefined>undefined
  }),

  actions: {
    logout() {
      this.$patch({ user: undefined })
    },

    async loadUser() {

      const user: Result<any, any> = await apiClient.get('api/me');
      if (user.ok) {
        this.$patch({ user: user.val });
      }
    },
  },
})

if (import.meta.hot) {
  import.meta.hot.accept(acceptHMRUpdate(useUserStore, import.meta.hot))
}
