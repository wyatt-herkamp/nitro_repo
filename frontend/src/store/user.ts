import httpCommon from '@/http-common';
import {User} from '@/types/user';
import {acceptHMRUpdate, defineStore} from 'pinia';

const useUserStore = defineStore({
  id: 'user',
  state: () => ({
    user: <User | undefined>undefined,
  }),

  actions: {
    logout() {
      this.$patch({user: undefined});
    },
    async getAccount() {
      httpCommon.apiClient.get('/api/me').then((response) => {
        if (response.status === 200) {
          this.$patch({user: response.data});
        }
      });
    },
  },

});

if (import.meta.hot) {
  import.meta.hot.accept(acceptHMRUpdate(useUserStore, import.meta.hot));
}
