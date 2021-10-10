

<template>
  <el-container direction="horizontal" style="border: 1px solid #eee">
    <el-main>
      <el-alert
        v-if="form.error.length != 0"
        :title="form.error"
        type="error"
        closable="false"
      />
      <el-form label-position="top" :model="form" label-width="120px">
        <el-form-item label="Name">
          <el-input disabled v-model="form.name" :placeholder="computedStorage.name"></el-input>
        </el-form-item>
        <el-form-item label="Public Name">
          <el-input disabled
            v-model="form.public_name"
            :placeholder="computedStorage.public_name"
          ></el-input>
        </el-form-item>

        <el-form-item>
          <!--Yeah, I know. But please don't judge -->
          <el-button
            :disabled="
              form.name == computedStorage.name &&
              form.public_name == computedStorage.public_name
            "
            type="primary"
            @click="onSubmit"
            >Update Storage</el-button
          >
        </el-form-item>
      </el-form>
    </el-main>
  </el-container>
</template>

<script lang="ts">
import axios from "axios";
import { AuthToken, BasicResponse } from "@/backend/Response";
import router from "@/router";
import http from "@/http-common";
import { computed, defineComponent, onMounted, ref } from "vue";
import { useCookie } from "vue-cookie-next";
import { useRouter } from "vue-router";

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
       return props.storage
    });
    return { form, computedStorage };
  },
  methods: {
    async onSubmit() {
      let newUser = {
        name: this.form.name,
        public_name: this.form.public_name,
      };
      let body = JSON.stringify(newUser);
      console.log(body);
      const res = await http.post("/api/admin/storages/update", body, {
        headers: {
          "Content-Type": "application/json",
          Authorization: "Bearer " + this.$cookie.getCookie("token"),
        },
      });
      if (res.status != 200) {
        console.log("Data" + res.data);
        return;
      }
      const result = res.data;
      let value = JSON.stringify(result);
      console.log(value);

      let response: BasicResponse<unknown> = JSON.parse(value);

      if (response.success) {
        this.form.error = "Unable to Update Storage";
      } else {
        this.form.error = "Unable to Update Storage";
      }
    },
  },
});
</script>
<style scoped>
</style>