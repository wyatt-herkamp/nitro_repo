<template>
  <nav class="flex flex-wrap bg-slate-800 p-6 m-1 roun">
    <a @click="changeViewValue('General')">
      <EditItem> General </EditItem>
    </a>

    <a @click="changeViewValue('Frontend')">
      <EditItem> Frontend </EditItem>
    </a>
    <a @click="changeViewValue('Security')">
      <EditItem> Security </EditItem>
    </a>
    <a @click="changeViewValue('Deploy')">
      <EditItem> Deploy Settings </EditItem>
    </a>
  </nav>
</template>

<script lang="ts">
import { defineComponent, ref } from "vue";
import { useRouter } from "vue-router";
import EditItem from "@/components/repo/edit/EditItem.vue";

export default defineComponent({
  components: { EditItem },
  setup() {},
    methods: {
    changeViewValue(value:string) {
      console.log(value)
      this.$emit("changeView", value);
    },
  },
});
</script>
<style scoped>
.end {
  margin-left: auto;
}
</style>
