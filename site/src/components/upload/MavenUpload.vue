<template>
  <div class="example-typescript">
    <h1>Repo Uploader</h1>
    <el-form label-position="top" :model="coordinates" label-width="120px">
      <el-form-item>
        <el-form-item label="Group ID">
          <el-input v-model="coordinates.groupID"></el-input>
        </el-form-item>
        <el-form-item label="Artifact ID">
          <el-input v-model="coordinates.artifactID"></el-input>
        </el-form-item>
        <el-form-item label="Version">
          <el-input v-model="coordinates.version"></el-input>
        </el-form-item>

        <el-form-item label="Generate Pom">
          <el-switch v-model="coordinates.generatePom" />
        </el-form-item>
      </el-form-item>
    </el-form>
    <div class="upload">
      <el-table :data="fileTable" style="width: 100%">
        <el-table-column prop="name" label="OG File name" width="360" />
        <el-table-column prop="newName" label="New Name" width="360">
          <template v-slot:default="scope">
            <el-input
              v-model="scope.row.newName"
              size="small"
              controls-position="right"
            />
          </template>
        </el-table-column>
        <el-table-column prop="extension" label="Extension" width="180">
          <template v-slot:default="scope">
            <el-input
              :disabled="true"
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
        </file-upload>
        <el-button
          type="primary"
          v-if="!upload || !upload.active"
          @click="submitUpload"
          @click.prevent="upload.active = true"
        >
          <i class="fa fa-arrow-up" aria-hidden="true"></i>
          Start Upload
        </el-button>
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
import {defineComponent, ref} from "vue";

import {useCookie} from "vue-cookie-next";
import {Repository} from "@/backend/Response";
import FileUpload from "../../../src/FileUpload.vue";
import {VueUploadItem} from "vue-upload-component";
import http from "@/http-common";

/**
 * How does the manual upload work?
 * Basically I let the backend do it's thing with one adition of accepting a bearer token instead of basic when doing put requests. This keeps the backend basically the same with not aditional changes
 * Then I accept files in the frontend and do put request simulating a query.
 */
export default defineComponent({
  props: {
    repo: {
      required: true,
      type: Object as () => Repository,
    },
    storage: {
      required: true,
      type: Object as () => string,
    },
  },
  setup() {
    interface FileTableValue {
      name: string;
      newName: string;
      extension: string;
    }
    const files = ref<VueUploadItem[]>([]);
    const upload = ref<InstanceType<typeof FileUpload> | null>(null);
    // File Name, FinalName, Extension
    const fileTable = ref<FileTableValue[]>([]);
    fileTable.value.pop();
    const cookie = useCookie();
    const coordinates = ref({
      groupID: "org.kakara",
      artifactID: "engine",
      version: "1.0-SNAPSHOT",
      generatePom: false,
    });
    function inputFile(
      newFile: VueUploadItem | undefined,
      oldFile: VueUploadItem | undefined
    ) {
      if (newFile && !oldFile) {
        // add

        console.log("add", newFile);
        fileTable.value.push({
          name: newFile.name as string,
          newName: newFile.name as string,
          extension: "",
        });
      }
      if (newFile && oldFile) {
        // update
        console.log("update", newFile);
      }
      if (!newFile && oldFile) {
        // TODO add removal from File Table
        // remove
        console.log("remove", oldFile);
      }
    }
    return { files, inputFile, upload, fileTable, coordinates, cookie };
  },
  methods: {
    async submitUpload() {
      for (const value of this.fileTable) {
        let file = this.files.filter(
          (file) => file.name === value.name
        )[0] as VueUploadItem;

        let path =
          "storages/" +
          this.storage +
          "/" +
          this.repo.name +
          "/" +
          this.coordinates.groupID.replaceAll(".", "/") +
          "/" +
          this.coordinates.artifactID +
          "/" +
          this.coordinates.version +
          "/" +
          value.newName;
        console.log(path);
        console.log(file.file?.size);
        let response = await http.put(path, file.file, {
          headers: {
            Authorization: "Bearer " + this.cookie.getCookie("token"),
          },
        });
        console.log(response);
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
