<template>
  <div class="flex flex-col bg-slate-700">
    <div class="flex flex-row p-2">
      <input
        v-on:keypress.enter="addPermission()"
        v-model="permissionBox"
        class="flex-grow rounded-sm text-black p-1 mr-2"
      />
      <box-icon class="cursor-pointer" name="plus-circle"></box-icon>
    </div>
    <ul class="h-24 text-left m-2 overflow-y-auto">
      <li
        class="ml-2 my-1 py-1 flex flex-row border-2 w-11/12"
        v-for="(permission, index) in permissions.permissions"
        v-bind:key="permission"
      >
        <box-icon
          @click="removePermission(index)"
          class="cursor-pointer"
          name="checkbox-minus"
        ></box-icon>

        <pre class="ml-1 text-left"><code>{{permission}}</code></pre>
      </li>
    </ul>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref } from "vue";
import { RepositoryPermissions } from "@/types/user";
export default defineComponent({
  props: {
    modelValue: {
      required: true,
      type: Object as () => RepositoryPermissions,
    },
  },
  setup(props, { emit }) {
    const permissions = ref(props.modelValue);
    const permissionBox = ref("");
    const handleChange = (): void => {
      emit("update:modelValue", permissions);
    };
    const addPermission = (): void => {
      permissions.value.permissions.push(permissionBox.value);
      permissionBox.value = "";
      handleChange();
    };
    const removePermission = (index: number): void => {
      permissions.value.permissions.splice(index, 1);
      handleChange();
    };
    return {
      permissions,
      handleChange,
      permissionBox,
      addPermission,
      removePermission,
    };
  },
});
</script>
