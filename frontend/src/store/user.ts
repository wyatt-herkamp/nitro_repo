import { computed, inject, reactive } from "vue";
import { User } from "nitro_repo-api-wrapper";
import { getUser } from "nitro_repo-api-wrapper";

const state = reactive({
  user: <User | undefined>undefined,
});

const getters = reactive({
  isLoggedIn: computed(() => state.user != undefined),
});
const actions = {
  async getUser() {
    if (state.user != undefined) {
      return;
    }
    const token: string | undefined = inject('token')
    if (token == undefined) {
      return;
    }
    const user = await getUser(token);
    if (user.err) return;
    state.user = user.val;
  },
};

export default { state, getters, ...actions };
