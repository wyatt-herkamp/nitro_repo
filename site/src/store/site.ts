import {reactive} from "vue";
import {useCookie} from "vue-cookie-next";
import {getSiteInfo, SiteInfo} from "@/backend/api/backend/Generic";


const state = reactive<SiteInfo>({
    name: "Nitro Repo Frontend",
    description: "Nitro Repo Frontend",
});


const actions = {
    async getSiteInfo() {
        const cookie = useCookie();
        if (!cookie.isCookieAvailable("site.name")) {
            let value = await getSiteInfo();
            if (value.ok) {
                cookie.setCookie("site.name", value.val.name);
                cookie.setCookie("site.description", value.val.description);
            } else {
                console.error("Unable to pull site info " + value.val)
                return;
            }
        }
        state.name = cookie.getCookie("site.name");
        state.description = cookie.getCookie("site.description");

    },
};

export default {state, ...actions};
