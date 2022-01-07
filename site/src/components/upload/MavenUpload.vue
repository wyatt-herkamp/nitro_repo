<template></template>
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
