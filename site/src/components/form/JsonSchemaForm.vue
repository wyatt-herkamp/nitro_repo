<template>
  <div>
    <form>
      <div
        v-for="input in inputs"
        :key="input.id">
        <label :for="input.id">{{ input.label }}</label>
        <component
          :is="input.component"
          v-bind="input.props"
          v-model="value[input.id]" />
      </div>
    </form>
  </div>
</template>

<script setup lang="ts">
import { EnumInput, SchemaForm, type FormInputType } from "nitro-jsf";
import { computed, type Component, type PropType } from "vue";
import TextInput from "./text/TextInput.vue";
import DropDown from "./dropdown/DropDown.vue";
import SwitchInput from "./SwitchInput.vue";

const props = defineProps({
  form: Object as PropType<SchemaForm>,
});
const value = defineModel<any>();

const inputs = computed(() => {
  return props.form
    ?.getProperties(value)
    .map((field) => {
      return formFieldToInput(field);
    })
    .filter((input) => input !== undefined);
});
interface Input {
  component: Component;
  label: string;
  id: string;
  props: Record<string, any>;
}
function formFieldToInput(field: FormInputType): Input | undefined {
  if (!value.value[field.key()]) {
    value.value[field.key()] = field.default();
  }
  switch (field.type()) {
    case "string":
      return {
        component: TextInput,
        label: field.title() ?? field.key(),
        id: field.key(),
        props: {},
      };
    case "enum": {
      const enumField = field as EnumInput;
      const options = enumField.values.map((value) => {
        return {
          label: value.title ?? value.value,
          value: value.value,
        };
      });
      console.log(options);
      return {
        component: DropDown,
        label: enumField.title() ?? enumField.key(),
        id: enumField.key(),
        props: {
          options: options,
        },
      };
    }
    case "boolean": {
      return {
        component: SwitchInput,
        label: field.title() ?? field.key(),
        id: field.key(),
        props: {},
      };
    }
    default:
      console.error(`Unsupported field type: ${field.type()}`);
      return undefined;
  }
}
</script>
<style scoped lang="scss">
form {
  display: flex;
  flex-direction: row;
  flex-wrap: wrap;
  gap: 2rem;
}
</style>
