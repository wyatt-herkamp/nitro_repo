<template>
  <div class="bg-slate-800 md:m-1 rounded-lg">
    <ul class="flex flex-wrap justify-start" ref="menu">
      <slot></slot>
    </ul>
  </div>
</template>

<script lang="ts">
import { defineComponent, provide, reactive, ref, watch } from "vue";

import { MenuProvider } from "./SubNavType";

export default defineComponent({
  props: {
    modelValue: {
      type: String,
      default: "",
    },
  },
  setup(props, { emit }) {
    const menu = ref<HTMLUListElement>();
    const activeIndex = ref<MenuProvider["activeIndex"]>(props.modelValue);
    const addItem: MenuProvider["addItem"] = (item) => {
      items.value[item.index] = item;
    };

    const removeItem: MenuProvider["removeItem"] = (item) => {
      delete items.value[item.index];
    };
    const onClick: MenuProvider["onClick"] = (item) => {
      activeIndex.value = item;
    };
    watch(activeIndex, () => {
      emit("update:modelValue", activeIndex.value);
    });
    const items = ref<MenuProvider["items"]>({});
    provide<MenuProvider>(
      "rootMenu",
      reactive({
        items,
        addItem,
        removeItem,
        onClick,
        activeIndex,
      })
    );
    return { menu };
  },
});
</script>
