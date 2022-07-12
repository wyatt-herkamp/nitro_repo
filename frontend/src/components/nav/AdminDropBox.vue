<template>
  <li>
    <div
      class="cursor-pointer text-left"
      @click="openAdminBox()"
      @keyup.space="openAdminBox()"
    >
      <span class="smItemHeader">Admin</span>
      <div class="icon">
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="24"
          height="24"
          style="fill: rgba(255, 255, 255, 1)"
          :class="adminDropDown ? 'rotateSVG' : ''"
        >
          <path
            d="m6.293 13.293 1.414 1.414L12 10.414l4.293 4.293 1.414-1.414L12 7.586z"
          ></path>
        </svg>
      </div>
    </div>
  </li>
  <ul ref="adminDropDownUI" :class="adminDropDown ? 'flex flex-col' : 'hidden'">
    <li>
      <router-link to="/admin" @click="close()" class="smItem"
        >Admin
      </router-link>
    </li>
    <li>
      <router-link to="/admin/users" @click="close()" class="smItem"
        >Users</router-link
      >
    </li>
    <li>
      <router-link to="/admin/storages" @click="close()" class="smItem"
        >Storages</router-link
      >
    </li>
  </ul>
</template>

<script lang="ts">
import { defineComponent, ref } from "vue";
import { useRouter } from "vue-router";

export default defineComponent({
  emits: ["clicked"],
  setup(props, { emit }) {
    const router = useRouter();
    const adminDropDownUI = ref<HTMLUListElement>();
    const adminDropDown = ref(false);

    const openAdminBox = (): void => {
      adminDropDown.value = !adminDropDown.value;
    };
    const close = (): void => {
      openAdminBox();
      emit("clicked", adminDropDown.value);
    };
    return {
      router,
      close,
      openAdminBox,
      adminDropDown,
      adminDropDownUI,
    };
  },
});
</script>
<style scoped>
.smItemHeader {
  @apply text-lg;
  @apply px-2;
  @apply py-4;
}
.smItem {
  @apply block;
  @apply text-sm;
  @apply text-left;

  @apply px-6;
  @apply py-4;
  @apply hover:bg-green-500;
  @apply transition;
  @apply duration-300;
}
.icon {
  @apply mt-2;
  @apply align-middle;
  @apply inline-block;
}
svg {
  @apply transition;

  @apply ease-in-out;
  @apply duration-300;
}
</style>
