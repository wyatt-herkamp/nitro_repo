<template>
  <div v-if="repository != undefined" class="min-h-screen w-full flex">
    <div class="flex flex-col w-full">
      <EditMenu @changeView="view = $event" />
      <h1 class="text-slate-50">
        {{ repository.storage }}/{{ repository.name }} Data: {{ view }}
      </h1>
      <div class="w-full content-start " v-if="view == 'General'">
        <form class=" content-start  max-w-lg">
          <div class="flex flex-wrap -mx-3 mb-6">
            <div class="w-full md:w-1/2 px-3 mb-6 md:mb-0">
              <label for="grid-name"> name </label>
              <input
                class="text-input"
                id="grid-name"
                type="text"
                v-model="repository.name"
                disabled
              />
            </div>
            <div class="w-full md:w-1/2 px-3">
              <label for="grid-Storage"> Storage </label>
              <input
                class="text-input"
                id="grid-Storage"
                type="text"
                v-model="repository.storage"
                disabled
              />
            </div>
            <div class="w-full md:w-1/2 px-3">
              <label for="grid-created"> Date Created </label>
              <input
                class="text-input"
                id="grid-created"
                type="text"
                v-model="date"
                disabled
              />
            </div>
            <div class="w-full md:w-1/2 px-3">
              <label for="grid-type"> Repo Type</label>
              <input
                class="text-input"
                id="grid-type"
                type="text"
                v-model="repository.repo_type"
                disabled
              />
            </div>
          </div>
        </form>
      </div>
      <div v-if="view == 'Frontend'"></div>
      <div v-if="view == 'Security'"></div>
      <div v-if="view == 'Deploy'"></div>
    </div>
    <div class="flex flex-col float-right w-auto bg-slate-800">
      <ViewRepo :repositoryType="repository" />
    </div>
  </div>
</template>
<style scoped>
.content{

}
label {
  @apply block;
  @apply uppercase;
  @apply tracking-wide;
  @apply text-white;
  @apply text-xs;
  @apply font-bold;
  @apply mb-2;
}
.text-input {
  @apply appearance-none;
  @apply block;
  @apply w-full;
  @apply bg-gray-200;
  @apply text-gray-700;
  @apply border;
  @apply border-gray-200;
  @apply rounded;
  @apply py-3;
  @apply px-4;
  @apply leading-tight;
  @apply focus:outline-none;
  @apply focus:bg-white;
  @apply focus:border-gray-500;
}
</style>
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
import EditMenu from "@/components/repo/edit/EditMenu.vue";
import ViewRepo from "@/components/repo/ViewRepo.vue";
import { defineComponent, ref } from "vue";
import { useCookie } from "vue-cookie-next";
import { useMeta } from "vue-meta";
import { useRoute, useRouter } from "vue-router";

export default defineComponent({
  components: { ViewRepo, EditMenu },
  setup() {
    const router = useRouter();
    const route = useRoute();
    let view = ref("General");
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
      view,
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
