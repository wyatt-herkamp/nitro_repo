<template>
  <li
    class=" py-1.5 rounded-lg mx-auto sm:mx-2 bg-slate-700 font-bold px-6 m-1 text-white"
    :class="active ? '' : 'cursor-pointer'"
    @click="handleClick()"
  >
    <slot></slot>
  </li>
</template>
<script lang="ts">
import {
  computed,
  defineComponent,
  inject,
  onBeforeUnmount,
  onMounted,
  reactive,
  ref,
} from "vue";
import { MenuItemType, MenuProvider } from "./SubNavType";

export default defineComponent({
  props: {
    index: {
      type: String,
      required: true,
    },
    disabled: {
      type: Boolean,
      default: false,
    },
  },
  setup(props, { emit }) {
    const rootMenu = inject<MenuProvider>("rootMenu") as MenuProvider;
    const active = computed(() => props.index === rootMenu.activeIndex);

    const item: MenuItemType = reactive({
      index: props.index,
      active: active.value,
    });
    const handleClick = () => {
      if (!props.disabled) {
        emit("click", item);
        rootMenu.onClick(item.index);
      }
    };

    onMounted(() => {
      rootMenu.addItem(item);
    });
    onBeforeUnmount(() => {
      rootMenu.removeItem(item);
    });

    return { handleClick, item, active };
  },
});
</script>