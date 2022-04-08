<template>
    <ul class="flex flex-wrap bg-slate-800 p-6 m-1" ref="menu">
      <slot></slot>
    </ul>
</template>

<script lang="ts">
import { defineComponent, onMounted, provide, reactive, ref, watch } from "vue";

import { apiURL } from "@/http-common";
import { MenuProvider } from "./SubNavType";

export default defineComponent({
  props: {
    modelValue: String,
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
