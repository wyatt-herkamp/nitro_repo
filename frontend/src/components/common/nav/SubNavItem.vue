<template>
  <li
    class="rounded-lg mx-2 bg-slate-700 p-2 m-3 text-white"
    :class="[
      [active ? 'active' : 'cursor-pointer'],
      [disabled ? 'cursor-default' : ''],
    ]"
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
    const active = computed(() => {
      return props.index === rootMenu.activeIndex;
    });

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
<style scoped>
.active {
  @apply font-bold;
}
</style>
