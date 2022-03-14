
<style scoped></style>
<template>
  <div v-if="storage != undefined" class="min-h-screen w-full">
    <h2 class="text-white m-3 text-left">Storage</h2>

    <div class="settingContent">
      <div class="settingBox">
        <label for="grid-name"> name </label>
        <input
          class="disabled"
          id="grid-name"
          type="text"
          v-model="storage.name"
          disabled
        />
      </div>
      <div class="settingBox">
        <label for="grid-public-name"> Public Name </label>
        <input
          class="disabled"
          id="grid-public-name"
          type="text"
          v-model="storage.public_name"
          disabled
        />
      </div>
      <div class="settingBox">
        <label for="grid-created"> Date Created </label>
        <input
          class="disabled"
          id="grid-created"
          type="text"
          v-model="date"
          disabled
        />
      </div>
      <div class="settingBox">
        <!-- Yes! A Model confirming the delete needs to happen. However right now I just need to delete something -->
        <button
          @click="deleteStorage"
          class="
            bg-blue-500
            hover:bg-blue-700
            text-white
            font-bold
            py-2
            px-4
            rounded
            m-5
          "
        >
          Delete Storage
        </button>
      </div>
    </div>
  </div>
</template>
<style scoped>
label {
  @apply block;
  @apply uppercase;
  @apply tracking-wide;
  @apply text-white;
  @apply text-xs;
  @apply font-bold;
  @apply text-left;
  @apply my-3;
}
.settingBox {
  @apply md:w-1/2;
  @apply px-3;
}
.disabled {
  @apply appearance-none;
  @apply block;
  @apply w-full;
  @apply bg-gray-300;
  @apply text-gray-700;
  @apply border;
  @apply border-gray-800;
  @apply rounded;
  @apply py-3;
  @apply px-4;
  @apply leading-tight;
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
.toggle-bg:after {
  content: "";
  @apply absolute top-0.5 left-0.5 bg-white border border-gray-300 rounded-full h-5 w-5 transition shadow-sm;
}

input:checked + .toggle-bg:after {
  transform: translateX(100%);
  @apply border-white;
}
.settingContent {
  @apply max-w-lg;
  @apply mx-auto;
}
input:checked + .toggle-bg {
  @apply bg-blue-600 border-blue-600;
}
</style>
<script lang="ts">
import { getStorage } from "@/backend/api/Storages";
import { defineComponent, ref } from "vue";
import { useCookie } from "vue-cookie-next";
import { useMeta } from "vue-meta";
import { useRoute, useRouter } from "vue-router";
import { Storage } from "@/backend/Response";
import { deleteStorage } from "@/backend/api/admin/Storage";
export default defineComponent({
  props: {
    storageId: {
      type: Number,
      required: true,
    },
  },
  setup(props) {
    let storage = ref<Storage | undefined>(undefined);
    let date = ref<string | undefined>(undefined);
    const cookie = useCookie();
    const { meta } = useMeta({
      title: "Nitro Repo",
    });

    const getStorageInternal = async () => {
      try {
        const value = (await getStorage(
          cookie.getCookie("token"),
          props.storageId
        )) as Storage;
        storage.value = value;
        date.value = new Date(storage.value.created).toLocaleDateString(
          "en-US"
        );
        meta.title = value.name;
      } catch (e) {
        console.log(e);
      }
    };
    getStorageInternal();

    return {
      date,
      storage,
    };
  },
  methods: {
    async deleteStorage() {
      if (this.storage == undefined) {
        return;
      }
      const response = await deleteStorage(
        this.storage.id,
        this.$cookie.getCookie("token")
      );
      if (response.ok) {
        this.$notify({
          title: "Storage Deleted",
          type: "success",
        });
        this.$router.push("/admin/storages");
      } else {
        this.$notify({
          title: "Unable to Delete Storage",
          text: JSON.stringify(response.val.user_friendly_message),
          type: "error",
        });
      }
    },
  },
});
</script>
