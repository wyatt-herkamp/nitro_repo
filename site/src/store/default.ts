import {reactive} from "vue";
import {BasicResponse} from "@/backend/Response";
import http from "@/http-common";

const state = reactive({
  installed: false,
});

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
  },
};
export default { state, ...actions };
