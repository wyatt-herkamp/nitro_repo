import { createStore } from "vuex";
import { computed, reactive } from 'vue'
import { BasicResponse, User } from '@/backend/Response'
import { getUser } from '@/backend/api/User';
import { useCookie } from 'vue-cookie-next'
import http from "@/http-common"
const state = reactive({
  installed: false,
})


const actions = {

  async init() {
    const installed = await http.get("/api/installed");
    if (installed.status != 200) {
      return;
    }
    const data = installed.data as BasicResponse<unknown>;
    if (data.success) {
      const data = installed.data as BasicResponse<boolean>;
      state.installed = data.data;

    }


  }
}
export default { state, ...actions }



