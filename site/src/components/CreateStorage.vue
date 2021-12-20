<template>
  <el-container direction="horizontal" style="border: 1px solid #eee">
    <el-main>
      <el-alert
        v-if="form.error.length != 0"
        :title="form.error"
        type="error"
      />
      <el-form label-position="top" :model="form" label-width="120px">
        <el-form-item label="Name">
          <el-input v-model="form.name"></el-input>
        </el-form-item>
        <el-form-item label="Public Name">
          <el-input v-model="form.public_name"></el-input>
        </el-form-item>

        <el-form-item>
          <el-button type="primary" @click="onSubmit"
            >Create New Storage</el-button
          >
        </el-form-item>
      </el-form>
    </el-main>
  </el-container>
</template>

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
