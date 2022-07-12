<template>
  <li class="link-box">
    <router-link
      class="link"
      :to="'/browse/' + file.full_path"
      v-if="file.directory"
    >
      <div>
        <div class="icon">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="24"
            height="24"
            style="fill: rgba(255, 255, 255, 0.5)"
          >
            <path
              d="M20 5h-8.586L9.707 3.293A.997.997 0 0 0 9 3H4c-1.103 0-2 .897-2 2v14c0 1.103.897 2 2 2h16c1.103 0 2-.897 2-2V7c0-1.103-.897-2-2-2zM4 19V7h16l.002 12H4z"
            ></path>
          </svg>
        </div>
        <span class="linkText">
          {{ file.name }}
        </span>
      </div>
    </router-link>
    <a
      class="link"
      :href="url + 'storages/' + file.full_path"
      v-if="!file.directory"
    >
      <div>
        <div class="icon">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="24"
            height="24"
            style="fill: rgba(255, 255, 255, 0.5)"
          >
            <path
              d="M18 22a2 2 0 0 0 2-2V8l-6-6H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12zM13 4l5 5h-5V4zM7 8h3v2H7V8zm0 4h10v2H7v-2zm0 4h10v2H7v-2z"
            ></path>
          </svg>
        </div>
        <span class="linkText">
          {{ file.name }}
        </span>
      </div>
    </a>
  </li>
</template>
<script lang="ts">
import { apiURL } from "@/http-common";
import { FileResponse } from "@nitro_repo/nitro_repo-api-wrapper";
import { defineComponent } from "vue";

export default defineComponent({
  props: {
    file: {
      type: Object as () => FileResponse,
      required: true,
    },
  },
  setup() {
    const url = apiURL;
    return { url };
  },
});
</script>
<style scoped>
.link {
  @apply block;
  @apply md:min-w-max;
  @apply p-3;
}
.linkText {
  @apply pl-2;
  @apply m-3;
  @apply align-middle;
  @apply text-quaternary;
}
.link-box {
  @apply my-1;
  @apply py-1;
  @apply mx-2;
  @apply rounded-lg;
  @apply bg-slate-800;
  @apply h-16;
  @apply transition;
  @apply ease-in-out;
  @apply duration-300;
  @apply hover:bg-slate-700;
  @apply md:relative;
}
.icon {
  @apply mt-2;
  @apply align-middle;
  @apply inline-block;
}
</style>
