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
import { SchemaForm, type FormInputType } from "nitro-jsf";
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
interface Option {
  label: string;
  value: string;
}

/**
 * Three separate nitro-jsf classes report `type() === "enum"`, and their `values` arrays have
 * different shapes. Narrowing structurally rather than with `instanceof` keeps this working whether
 * or not the installed version re-exports the classes themselves.
 *
 *  - `EnumInput`         — a plain `enum`/`oneOf`-of-strings property: `{ value, title }`
 *  - `AdjacentEnumInput` — `#[serde(tag, content)]`: `{ keyProperty, value, title }`
 *  - `InternalEnumInput` — externally tagged `{"Variant": {...}}`: `{ variantKey, value, title }`
 */
function enumOptions(field: FormInputType): Option[] {
  const values = (field as unknown as { values?: unknown[] }).values ?? [];
  return values.flatMap((entry) => {
    const record = entry as Record<string, unknown>;
    // The variant name lives under a different key per class; for a plain enum it is the value.
    const name =
      "keyProperty" in record
        ? record.keyProperty
        : "variantKey" in record
          ? record.variantKey
          : record.value;
    if (typeof name !== "string") {
      return [];
    }
    const title = record.title;
    return [{ label: typeof title === "string" ? title : name, value: name }];
  });
}

function formFieldToInput(field: FormInputType): Input | undefined {
  const key = field.key();
  if (typeof key !== "string") {
    // `InternalEnumInput.key()` returns one entry per variant. There is no single model property to
    // bind such an input to, so rendering it would write to a `"a,b,c"` key.
    console.error(`Unsupported multi-key field: ${key.join(", ")}`);
    return undefined;
  }
  // `=== undefined`, not a falsy check: with `!value.value[key]` a `false` stored against a field
  // whose default is `true` re-seeded the default on every render, so switches like
  // `yanking_allowed` could not be turned off.
  if (value.value[key] === undefined) {
    value.value[key] = field.default();
  }
  switch (field.type()) {
    case "string":
      return {
        component: TextInput,
        label: field.title() ?? key,
        id: key,
        props: {},
      };
    case "enum": {
      return {
        component: DropDown,
        label: field.title() ?? key,
        id: key,
        props: {
          options: enumOptions(field),
        },
      };
    }
    case "boolean": {
      return {
        component: SwitchInput,
        label: field.title() ?? key,
        id: key,
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
