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
        <el-form-item label="Repository Type">
          <el-select
            v-model="form.type"
            placeholder="Please select your Repo Type"
          >
            <el-option label="Maven" value="maven"></el-option>
            <el-option label="NPM" value="npm"></el-option>
          </el-select>
        </el-form-item>
        <el-form-item label="Storage">
          <el-select
            v-model="form.storage"
            placeholder="Please select your storage"
          >
            <el-option
              v-for="storage in storages.storages"
              :key="storage.id"
              :label="storage.public_name"
              :value="storage.name"
            ></el-option>
          </el-select>
        </el-form-item>
        <el-form-item>
          <el-button type="primary" @click="onSubmit"
            >Create Repository</el-button
          >
        </el-form-item>
      </el-form>
    </el-main>
  </el-container>
</template>

<script lang="ts">
import {DEFAULT_STORAGE_LIST, Repository,} from "@/backend/Response";
import {defineComponent, ref} from "vue";
import {useCookie} from "vue-cookie-next";
import {getStorages} from "@/backend/api/Storages";
import {createNewRepository} from "@/backend/api/admin/Repository";

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
      storage: "",
      type: "",
      error: "",
    });
    const cookie = useCookie();
    const isLoading = ref(false);

    const error = ref("");
    let storages = ref(DEFAULT_STORAGE_LIST);
    const getStorage = async () => {
      isLoading.value = true;
      try {
        const value = await getStorages(cookie.getCookie("token"));
        storages.value = value;

        isLoading.value = false;
      } catch (e) {
        error.value = "Error";
      }
    };
    getStorage();
    return {
      form,
      storages,
      isLoading,
      error,
      getStorage,
    };
  },
  methods: {
    async onSubmit() {
      const response = await createNewRepository(
        this.form.name,
        this.form.storage,
        this.form.type,
        this.$cookie.getCookie("token")
      );
      if (response.ok) {
        let data = response.val as Repository;
        this.$props.updateList(data.id);
        this.$notify({
          title: "Repository Created",
          type: "success",
        });
      } else {
        this.$notify({
          title: "Unable to Create Repository",
          text: JSON.stringify(response.val.user_friendly_message),
          type: "error",
        });
      }
    },
  },
});
</script>
<style scoped></style>
