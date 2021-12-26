<template>

</template>
<style scoped></style>
<script lang="ts">
import {DEFAULT_REPO, Repository, RepositoryListResponse,} from "@/backend/Response";
import {apiURL} from "@/http-common";
import {defineComponent, ref} from "vue";
import {useCookie} from "vue-cookie-next";
import {useRouter} from "vue-router";
import {getRepoByID} from "@/backend/api/Repository";
import {
  setActiveStatus,
  setPolicy,
  setVisibility,
  updateBadge,
  updateDeployReport,
  updateFrontend,
} from "@/backend/api/admin/Repository";

export default defineComponent({
  props: {
    repo: {
      required: true,
      type: Object as () => RepositoryListResponse,
    },
  },

  setup(props) {
    const router = useRouter();
    const options = ref([
      { value: "DeployerUsername", label: "Deploy Username" },
      { value: "Time", label: "Time" },
    ]);
    let repository = ref<Repository>(DEFAULT_REPO);
    let date = ref<string | undefined>(undefined);
    const cookie = useCookie();
    const isLoading = ref(false);
    const activeName = ref("general");
    const exampleBadgeURL = ref("");


    console.log(apiURL);
    const getRepo = async () => {
      isLoading.value = true;
      try {
        const value = (await getRepoByID(
          cookie.getCookie("token"),
          props.repo.id
        )) as Repository;
        repository.value = value;
        exampleBadgeURL.value =
            apiURL +
            "/badge/" +
            props.repo.storage +
            "/" +
          props.repo.name +
          "/nitro_repo_example/badge.svg";
        date.value = new Date(repository.value.created).toLocaleDateString(
          "en-US"
        );
        isLoading.value = false;
      } catch (e) {
        console.log(e);
      }
    };
    getRepo();

    return {
      isLoading,
      activeName,
      date,
      exampleBadgeURL,
      repository,
      router,
      options,
    };
  },
  methods: {
    handleClick(tab: any, event: any) {
      console.log(tab.paneName);
      if (tab.paneName === "upload") {
        this.router.replace(
          "/upload/" + this.storage.name + "/" + this.repo.name
        );
      }
    },

    async updateActiveStatus() {
      if (this.repository.id == 0) {
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
      if (this.repository.id == 0) {
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
      if (this.repository.id == 0) {
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
      if (this.repository.id == 0) {
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
      if (this.repository.id == 0) {
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
