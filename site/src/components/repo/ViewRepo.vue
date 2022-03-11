<template>
  <div v-if="repository != undefined">
    <h1 class="text-slate-50">{{ repository.storage }}/{{ repository.name }}</h1>
  </div>
</template>
<style scoped></style>
<script lang="ts">
import {
  setActiveStatus,
  setPolicy,
  setVisibility,
  updateBadge,
  updateDeployReport,
  updateFrontend,
} from "@/backend/api/admin/Repository";
import { getRepoByID } from "@/backend/api/Repository";
import { Repository } from "@/backend/Response";
import SideBar from "@/components/admin/SideBar.vue";
import { defineComponent, ref } from "vue";
import { useCookie } from "vue-cookie-next";
import { useMeta } from "vue-meta";
import { useRoute, useRouter } from "vue-router";

export default defineComponent({
  components: { SideBar },
  setup() {
    const router = useRouter();
    const route = useRoute();

    const options = ref([
      { value: "DeployerUsername", label: "Deploy Username" },
      { value: "Time", label: "Time" },
    ]);
    let repository = ref<Repository | undefined>(undefined);
    let date = ref<string | undefined>(undefined);
    const cookie = useCookie();
    const isLoading = ref(false);
    const exampleBadgeURL = ref("");
    const repoID = route.params.repo as string;
    console.log(repoID);
    const { meta } = useMeta({
      title: "Nitro Repo",
    });

    const getRepo = async () => {
      try {
        const value = (await getRepoByID(
          cookie.getCookie("token"),
          Number.parseInt(repoID)
        )) as Repository;
        repository.value = value;
        date.value = new Date(repository.value.created).toLocaleDateString(
          "en-US"
        );
        meta.title = value.name;
      } catch (e) {
        console.log(e);
      }
    };
    getRepo();

    return {
      date,
      exampleBadgeURL,
      repository,
      router,
      options,
    };
  },
  methods: {
    handleClick(tab: any, event: any) {
      if (this.repository == undefined) return;
      if (tab.paneName === "upload") {
        this.router.replace(
          "/upload/" + this.repository.storage + "/" + this.repository.name
        );
      }
    },

    async updateActiveStatus() {
      if (this.repository == undefined) {
        this.$notify({
          title: "Unable Update Repository",
          text: "Repository is still undefined",
          type: "error",
        });
        return;
      }

      const response = await setActiveStatus(
        this.repository.id,
        this.repository.settings.active,
        this.$cookie.getCookie("token")
      );
      if (response.ok) {
        this.$notify({
          title: "Updated Repository",
          type: "info",
        });
      } else {
        this.$notify({
          title: "Unable Update Repository",
          text: JSON.stringify(response.val.user_friendly_message),
          type: "error",
        });
      }
    },
    async updatePolicy() {
      if (this.repository == undefined) {
        this.$notify({
          title: "Unable Update Repository",
          text: "Repository is still undefined",
          type: "error",
        });
        return;
      }
      const response = await setPolicy(
        this.repository.id,
        this.repository.settings.policy,
        this.$cookie.getCookie("token")
      );
      if (response.ok) {
        this.$notify({
          title: "Updated Repository",
          type: "info",
        });
      } else {
        this.$notify({
          title: "Unable Update Repository",
          text: JSON.stringify(response.val.user_friendly_message),
          type: "error",
        });
      }
    },
    async updateVisibility() {
      if (this.repository == undefined) {
        this.$notify({
          title: "Unable Update Repository",
          text: "Repository is still undefined",
          type: "error",
        });
        return;
      }
      const response = await setVisibility(
        this.repository.id,
        this.repository.security.visibility,
        this.$cookie.getCookie("token")
      );
      if (response.ok) {
        console.log(response.val.security.visibility);
        this.$notify({
          title: "Updated Repository",
          type: "info",
        });
      } else {
        this.$notify({
          title: "Unable Update Repository",
          text: JSON.stringify(response.val.user_friendly_message),
          type: "error",
        });
      }
    },
    async submitFrontend() {
      if (this.repository == undefined) {
        this.$notify({
          title: "Unable Update Repository",
          text: "Repository is still undefined",
          type: "error",
        });
        return;
      }
      {
        const response = await updateFrontend(
          this.repository.id,
          this.repository.settings.frontend.enabled,
          this.repository.settings.frontend.page_provider,
          this.$cookie.getCookie("token")
        );
        if (response.ok) {
          console.log(response.val.security.visibility);
          this.$notify({
            title: "Updated Frontend",
            type: "info",
          });
        } else {
          this.$notify({
            title: "Unable Update Repository",
            text: JSON.stringify(response.val.user_friendly_message),
            type: "error",
          });
        }
      }
      {
        let response = await updateBadge(
          this.repository.id,
          this.repository.settings.badge.style,
          this.repository.settings.badge.label_color,
          this.repository.settings.badge.color,
          this.$cookie.getCookie("token")
        );
        if (response.ok) {
          console.log(response.val.security.visibility);
          this.$notify({
            title: "Updated Badge",
            type: "info",
          });
        } else {
          this.$notify({
            title: "Unable Update Repository",
            text: JSON.stringify(response.val.user_friendly_message),
            type: "error",
          });
        }
      }
    },
    async updateReport() {
      if (this.repository == undefined) {
        this.$notify({
          title: "Unable Update Repository",
          text: "Repository is still undefined",
          type: "error",
        });
        return;
      }

      const response = await updateDeployReport(
        this.repository.id,
        this.repository.deploy_settings.report_generation.active,
        this.repository.deploy_settings.report_generation.values,
        this.$cookie.getCookie("token")
      );
      if (response.ok) {
        console.log(response.val.security.visibility);
        this.$notify({
          title: "Updated Report Settings",
          type: "info",
        });
      } else {
        this.$notify({
          title: "Unable Update Repository",
          text: JSON.stringify(response.val.user_friendly_message),
          type: "error",
        });
      }
    },
  },
});
</script>
