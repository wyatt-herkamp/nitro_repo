<template>
  <div class="w-full">
    <div
      class="bg-slate-800 shadow-md rounded-lg px-3 py-2 mb-4 max-w-screen-xl"
    >
      <div id="header">
        <slot name="title"></slot>
      </div>
      <div class="flex flex-row">
        <div class="md:w-3/4 inline-block">
          <div class="searchSection">
            <div class="pl-2 pt-2">
              <box-icon name="search-alt-2"></box-icon>
            </div>
            <input id="search" placeholder="Search Here" type="text" />
          </div>
        </div>

        <div class="createSection inline-block">
          <slot name="createButton"></slot>
        </div>
      </div>
      <div>
        <ul ref="core">
          <li v-for="value in modelValue" :key="value.name">
            <router-link
              v-if="value.goTo != undefined"
              :to="value.goTo"
              class="routerLink"
            >
              <div class="px-1">{{ value.name }}</div>
            </router-link>
          </li>
        </ul>
      </div>
    </div>
  </div>
</template>
<style scoped>
#header {
  @apply block;
  @apply text-slate-50;
  @apply text-lg;
  @apply text-left;
  @apply font-semibold;
  @apply py-2;
  @apply px-2;
}
.routerLink {
  @apply cursor-pointer;
  @apply py-2;
  @apply text-slate-50;
  @apply flex flex-row;
  @apply m-1;
  @apply hover:translate-x-2;
  @apply transition-transform;
  @apply ease-in;
  @apply duration-200;
}
.searchSection {
  @apply flex;

  @apply items-center;
  @apply bg-gray-200;
  @apply rounded-md;
  @apply h-max;
  @apply my-2;
}
.createSection {
  @apply flex;
  @apply rounded-md;
  @apply w-1/4;
  @apply my-1;
}
#search {
  @apply w-3/4;
  @apply rounded-md;
  @apply bg-gray-200;
  @apply text-gray-700;
  @apply leading-tight;
  @apply focus:outline-none;
}
</style>
<script lang="ts">
import { defineComponent, nextTick, onMounted, ref, watch } from "vue";
import { ListItem } from "./ListTypes";

export default defineComponent({
  props: {
    modelValue: {
      type: Object as () => ListItem[],
      required: true,
    },
  },
  setup(props, { emit }) {
    const core = ref<HTMLUListElement>();
    console.log(props.modelValue.length);
    return { core };
  },
});
</script>
error,
