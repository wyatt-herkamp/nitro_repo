<template>
  <li class="rounded-lg mx-2 bg-slate-700 font-bold m-3 text-white">
    <router-link
      :to="href"
      class="flex flex-row items-center h-12 text-slate-50 hover:text-slate-100"
    >
      <span class="inline-flex items-center justify-center w-12">
        <box-icon :name="icon"></box-icon>
      </span>
      <span
        :class="[active ? 'active' : '']"
        class=" text-sm font-medium mr-2"
        >{{ name }}</span
      >
    </router-link>
  </li>
</template>
<script lang="ts">
import {
  computed,
  defineComponent,
  inject,
  onBeforeUnmount,
  onMounted,
  provide,
  reactive,
  ref,
  watch,
} from "vue";
import { MenuItemType, MenuProvider } from "./SubNavType";

export default defineComponent({
  props: {
    icon: {
      required: true,
      type: String,
    },
    href: {
      required: true,
      type: String,
    },
    name: {
      required: true,
      type: String,
    },
  },
  setup(props, { emit }) {
    const rootMenu = inject<MenuProvider>("rootMenu") as MenuProvider;
    const active = computed(() => {
      console.log(props.name);
      console.log(rootMenu.activeIndex);
      return props.name.toLocaleLowerCase() === rootMenu.activeIndex;
    });

    const item: MenuItemType = reactive({
      index: props.name,
      active: active.value,
    });

    onMounted(() => {
      rootMenu.addItem(item);
    });
    onBeforeUnmount(() => {
      rootMenu.removeItem(item);
    });

    return { item, active };
  },
});
</script>
<style scoped>
.active {
  @apply font-bold;
}
.notActive {
}
</style>