<template>
  <div class="example-typescript">
    <h1>Repo Uploader</h1>
    <div class="upload">
      <el-table :data="fileTable" style="width: 100%">
        <el-table-column prop="name" label="OG File name" width="180" />
        <el-table-column prop="newName" label="New Name" width="180">
          <template v-slot:default="scope">
            <el-input
              v-model="scope.row.newName"
              size="small"
              controls-position="right"
            />
          </template>
        </el-table-column>
        <el-table-column prop="extension" label="Extension">
          <template v-slot:default="scope">
            <el-input
              v-model="scope.row.extension"
              size="small"
              controls-position="right"
            />
          </template>
        </el-table-column>
      </el-table>
      <div class="example-btn">
        <file-upload
          class="btn btn-primary"
          :multiple="true"
          :size="1024 * 1024 * 10"
          v-model="files"
          @input-file="inputFile"
          ref="upload"
        >
          <br />

          <el-button type="primary">Select File</el-button>
          <el-button
            type="primary"
            v-if="!upload || !upload.active"
            @click="submitUpload"
            @click.prevent="upload.active = true"
          >
            <i class="fa fa-arrow-up" aria-hidden="true"></i>
            Start Upload
          </el-button>
        </file-upload>
      </div>
    </div>
  </div>
</template>
<style>
.example-typescript label.btn {
  margin-bottom: 0;
  margin-right: 1rem;
}
</style>


<script lang="ts">
import { defineComponent } from "vue";

import { useCookie } from "vue-cookie-next";
import { getUsers } from "@/backend/api/User";
import { DEFAULT_USER_LIST, Repository } from "@/backend/Response";
import { ref, SetupContext } from "vue";
import FileUpload from "../../../src/FileUpload.vue";
import { VueUploadItem } from "vue-upload-component";
export default defineComponent({
  props: {
    repo: {
      required: true,
      type: Object as () => Repository,
    },
  },
  setup() {
    const files = ref<any[]>([]);
    const upload = ref<InstanceType<typeof FileUpload> | null>(null);
    // File Name, FinalName, Extension
    const fileTable = ref([{}]);
    fileTable.value.pop();
    function inputFile(
      newFile: VueUploadItem | undefined,
      oldFile: VueUploadItem | undefined
    ) {
      if (newFile && !oldFile) {
        // add
        console.log("add", newFile);
        fileTable.value.push({
          name: newFile.name as string,
          newName: newFile.name,
          extension: "",
        });
      }
      if (newFile && oldFile) {
        // update
        console.log("update", newFile);
      }
      if (!newFile && oldFile) {
        // remove
        console.log("remove", oldFile);
      }
    }
    return { files, inputFile, upload, fileTable };
  },
  methods: {
    submitUpload() {
      for (const value of this.files) {
        console.log(value);
      }
    },
    addFile(file: any, fileList: any) {
      this.files.push();
    },
  },
});
</script>

<style>
.el-menu-vertical-demo:not(.el-menu--collapse) {
  width: 200px;
  min-height: 400px;
}
</style>
