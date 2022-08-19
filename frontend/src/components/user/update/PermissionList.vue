<template>
  <div v-bind="$attrs" class="flex flex-col bg-secondary">
    <h1 class="text-quaternary m-2 text-lg"><slot /></h1>
    <div class="relative w-full">
      <div class="flex flex-row p-2">
        <input
          v-on:keypress.enter="
            addPermission();
            handleShowSuggestion(false);
          "
          placeholder="{storage_name}/{repository_name}"
          v-model="permissionBox"
          ref="permssionBoxRef"
          type="text"
          class="flex-grow rounded-sm text-black p-2 mr-2"
          v-on:focusin="showSuggestion = true"
          v-on:focusout="handleShowSuggestion(false)"
        />
      </div>
      <ul
        v-if="showSuggestion"
        class="absolute bg-white rounded-md border border-gray-100 w-11/12 mt-2 mx-2 text-black"
      >
        <li
          v-for="suggestion in getSuggestions()"
          v-bind:key="suggestion.value"
          class="pl-2 pr-2 py-1 border-gray-100 relative"
          :class="
            suggestion.readonly ? '' : 'cursor-pointer  hover:bg-gray-50 '
          "
          @click="maybePushPermission(suggestion)"
          :title="suggestion.hint"
        >
          {{ suggestion.value }}
          <span class="text-black/50 float-right" v-show="suggestion.hint">
            {{ suggestion.hint }}</span
          >
        </li>
      </ul>
    </div>
    <ul class="h-24 text-left m-2 overflow-y-auto">
      <li
        class="ml-2 my-1 py-1 flex flex-row border-2 w-11/12 cursor-pointer"
        v-for="(permission, index) in permissions.permissions"
        v-bind:key="permission"
        title="Left Click Edit. Right Click Delete"
        @click.left="
          permissionBox = permission;
          removePermission(index);
        "
        @click.right="removePermission(index)"
      >
        <pre class="ml-1 text-left"><code>{{permission}}</code></pre>
      </li>
    </ul>
  </div>
</template>

<script lang="ts">
import { RepositoryPermissions } from "@/types/userTypes";
import { defineComponent, nextTick, ref } from "vue";
import { notify } from "@kyvg/vue3-notification";

export default defineComponent({
  props: {
    modelValue: {
      required: true,
      type: Object as () => RepositoryPermissions,
    },
  },
  setup(props, { emit }) {
    const permissions = ref(props.modelValue);
    const showSuggestion = ref(false);
    const permissionBox = ref("");
    const permssionBoxRef = ref<HTMLInputElement | undefined>(undefined);
    const handleChange = (): void => {
      emit("update:modelValue", permissions);
    };
    const addPermission = (): void => {
      if (permissionBox.value.length === 0) {
        return;
      }
      if (permissions.value.permissions.includes(permissionBox.value)) {
        notify({
          type: "info",
          title: "Permission Already Exists",
        });
        return;
      }
      permissions.value.permissions.push(permissionBox.value);
      permissionBox.value = "";
      handleChange();
    };
    const removePermission = (index: number): void => {
      permissions.value.permissions.splice(index, 1);
      handleChange();
    };
    const handleShowSuggestion = (v: boolean): void => {
      setTimeout(() => {
        showSuggestion.value = v;
      }, 128);
    };
    return {
      permissions,
      handleChange,
      permissionBox,
      addPermission,
      removePermission,
      showSuggestion,
      permssionBoxRef,
      handleShowSuggestion,
    };
  },
  methods: {
    maybePushPermission(suggestion: {
      value: string;
      readonly: boolean;
    }): void {
      this.showSuggestion = false;
      if (!suggestion.readonly) {
        this.permissionBox = suggestion.value;
      }
    },
    getSuggestions: (): {
      value: string;
      readonly: boolean;
      hint?: string;
    }[] => {
      return [
        {
          value: "{storage_name}/{repository_name}",
          readonly: true,
        },
        {
          value: "{storage_name}/*",
          readonly: true,
          hint: "All repositories in the storage",
        },
        {
          value: "*",
          hint: "All repositories",
          readonly: false,
        },
      ];
    },
  },
});
</script>
