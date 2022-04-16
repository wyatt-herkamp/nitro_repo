import { getUser, User } from '@nitro_repo/nitro_repo-api-wrapper';
import { defineStore, acceptHMRUpdate } from 'pinia'
import { inject } from 'vue';

export const useUserStore = defineStore({
  id: 'user',
  state: () => ({
    user: <User | undefined>undefined,
    date: new Date()
  }),

  actions: {
    logout() {
      this.$patch({ user: undefined, date: new Date() })
    },

    async loadUser() {
      const token: string | undefined = inject('token')
      if (token == undefined) {
        return;
      }

      const user = await getUser(token);
      if (user.err) return;
      this.$patch({
        user: user.val,
        date: new Date(user.val.created).toLocaleDateString("en-US")
      })
    },
  },
})

if (import.meta.hot) {
  import.meta.hot.accept(acceptHMRUpdate(useUserStore, import.meta.hot))
}
