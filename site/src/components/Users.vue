<template>
  <div class="w-full">
    <div class="flex p-4">
      <div class="w-full float-left">
        <div class="bg-slate-800 shadow-md rounded-lg px-3 py-2 mb-4">
          <div class="block text-slate-50 text-lg font-semibold py-2 px-2">
            Users
          </div>
          <div class="flex flex-row">
            <div class="flex items-center bg-gray-200 rounded-md w-full h-max">
              <div class="pl-2">
                <svg
                    class="fill-current text-gray-500 w-6 h-6"
                    viewBox="0 0 24 24"
                    xmlns="http://www.w3.org/2000/svg"
                >
                  <path
                      class="heroicon-ui"
                      d="M16.32 14.9l5.39 5.4a1 1 0 0 1-1.42 1.4l-5.38-5.38a8 8 0 1 1 1.41-1.41zM10 16a6 6 0 1 0 0-12 6 6 0 0 0 0 12z"
                  />
                </svg>
              </div>
              <input
                  id="search"
                  class="
                  w-full
                  rounded-md
                  bg-gray-200
                  text-gray-700
                  leading-tight
                  focus:outline-none
                  py-2
                  px-2
                "
                  placeholder="User"
                  type="text"
              />
            </div>
            <CreateUser>
              <template v-slot:button>
                <button
                    class="
                    relative
                    inline-flex
                    items-center
                    justify-center
                    px-10
                    overflow-hidden
                    font-mono font-medium
                    tracking-tighter
                    text-white
                    bg-gray-800
                    rounded-lg
                    group
                  "
                >
                  <span
                      class="
                      absolute
                      w-0
                      h-max
                      transition-all
                      duration-500
                      ease-out
                      bg-slate-900
                      rounded-full
                      group-hover:w-56 group-hover:h-56
                    "
                  ></span>
                  <span
                      class="
                      absolute
                      inset-0
                      w-full
                      -mt-1
                      rounded-lg
                      opacity-30
                      bg-gradient-to-b
                      from-transparent
                      via-transparent
                      to-gray-700
                    "
                  ></span>
                  <span class="relative">Create User</span>
                </button>
              </template>
            </CreateUser>
          </div>
          <div>
            <ul v-if="users != undefined">
              <li v-for="user in users.users" :key="user.id">
                <router-link
                    :to="'/admin/user/' + user.id"
                    class="
                    cursor-pointer
                    py-2
                    text-slate-50
                    flex flex-row
                    m-1
                    hover:translate-x-2
                    transition-transform
                    ease-in
                    duration-200
                  "
                >
                  <div class="px-1">{{ user.name }}</div>
                </router-link>
              </li>
            </ul>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import {defineComponent, ref} from "vue";
import CreateUser from "@/components/CreateUser.vue";
import UpdateUser from "@/components/UpdateUser.vue";
import {useCookie} from "vue-cookie-next";
import {getUsers} from "@/backend/api/User";
import {DEFAULT_USER_LIST} from "@/backend/Response";

export default defineComponent({
  components: { CreateUser, UpdateUser },

  setup() {
    const cookie = useCookie();
    const isLoading = ref(false);

    let error = ref("");
    let users = ref(DEFAULT_USER_LIST);
    const getUser = async () => {
      isLoading.value = true;
      try {
        const value = await getUsers(cookie.getCookie("token"));
        users.value = value;

        isLoading.value = false;
      } catch (e) {
        error.value = "Error Loading";
      }
    };
    getUser();
    return {
      users,
      isLoading,
      error,
      getUser,
      cookie,
    };
  },
  methods: {
    updateList(id: number) {
      const getUser = async () => {
        try {
          const value = await getUsers(this.cookie.getCookie("token"));
          this.users = value;
          this.index = id;
        } catch (e) {
          this.error = "Error Loading";
        }
      };
      getUser();
    },
  },
});
</script>

<style>
.el-menu-vertical-demo:not(.el-menu--collapse) {
  width: 200px;
  min-height: 400px;
}
</style>
