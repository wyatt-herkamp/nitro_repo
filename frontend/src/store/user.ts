import {computed, reactive} from "vue";
import {User} from "nitro_repo-api-wrapper";
import {getUser} from "nitro_repo-api-wrapper";
import {useCookie} from "vue-cookie-next";

export const ANON_USER: User = {
  id: 0,
  name: "ANON",
  username: "ANON",
  email: "anon@example.com",
  permissions: {
    admin: false,
    deployer: false,
  },
  created: 0,
};
const state = reactive({
  user: ANON_USER,
});

const getters = reactive({
  isLoggedIn: computed(() => state.user.id !== 0),
});
const actions = {
  async getUser() {
    const cookie = useCookie();
    const token = cookie.getCookie("token");
    if (token == null) {
      return;
    }
    const user = await getUser(token);
    if (user == null) return;
    state.user = user;
  },
};

export default { state, getters, ...actions };
