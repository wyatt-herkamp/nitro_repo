<template></template>

<script lang="ts">
import {computed, defineComponent, ref} from "vue";

export default defineComponent({
  props: {
    storage: {
      required: true,
      type: Object as () => Storage,
    },
  },

  setup(props) {
    let form = ref({
      name: props.storage.name,
      public_name: props.storage.public_name,
      error: "",
    });
    const computedStorage = computed(() => {
      return props.storage;
    });
    return { form, computedStorage };
  },
  methods: {
    async onSubmit() {
      console.log("Ouch");
    },
  },
});
</script>
<style scoped></style>
