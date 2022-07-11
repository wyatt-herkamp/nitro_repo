<template></template>

<script lang="js">
import { defineComponent} from "vue";

export default defineComponent({
  props: {
    icon: {
      required: true,
      type: String,
    },
  },
  setup(props) {
    import { siSimpleicons } from 'simple-icons/icons';

    siSimpleicons.get

  },
});
</script>
