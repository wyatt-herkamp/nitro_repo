<template></template>

<script lang="ts">
import {Storage} from "@/backend/Response";
import {defineComponent, ref} from "vue";
import {createNewStorage} from "@/backend/api/admin/Storage";

export default defineComponent({
  props: {
    updateList: {
      required: true,
      type: Function,
    },
  },
  setup() {
    let form = ref({
      name: "",
      public_name: "",
      error: "",
    });
    return { form };
  },
  methods: {
    async onSubmit() {
      const response = await createNewStorage(
        this.form.name,
        this.form.public_name,
        this.$cookie.getCookie("token")
      );
      if (response.ok) {
        let data = response.val as Storage;
        this.$props.updateList(data.id);
        this.$notify({
          title: "Storage Created",
          type: "success",
        });
      } else {
        this.$notify({
          title: "Unable to Create Storage",
          text: JSON.stringify(response.val.user_friendly_message),
          type: "error",
        });
      }
    },
  },
});
</script>
<style scoped></style>
