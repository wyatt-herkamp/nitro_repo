<template>
  <el-menu
    :aria-expanded="true"
    default-active="0"
    class="el-menu-demo"
    mode="horizontal"
  >
    <el-menu-item @click="tab = 0" index="0">General Settings</el-menu-item>
    <el-menu-item @click="tab = 1" index="1">Email</el-menu-item>
  </el-menu>
  <div v-if="tab == 0">
    <el-form label-position="top" :model="settingForm" label-width="120px">
      <el-form-item>
        <el-form-item label="Installed">
          <el-input disabled v-model="unchangeable.installed"></el-input>
        </el-form-item>
        <el-form-item label="Version">
          <el-input disabled v-model="unchangeable.version"></el-input>
        </el-form-item>
        <el-form-item label="Name">
          <el-input v-model="settingForm['name.public']"></el-input>
        </el-form-item>
        <el-button type="primary" @click="onSettingSubmit"
          >Update Settings</el-button
        >
      </el-form-item>
    </el-form>
  </div>
  <div v-if="tab == 1">
    <el-form label-position="top" :model="email" label-width="120px">
      <el-form-item>
        <el-form-item label="Email Host">
          <el-input v-model="email['email.host']"></el-input>
        </el-form-item>
        <el-form-item label="Email Port">
          <el-input v-model="email['email.port']"></el-input>
        </el-form-item>
        <el-form-item label="Email Type">
          <el-select v-model="email['email.encryption']">
            <el-option label="TLS" value="TLS"></el-option>
            <el-option label="NONE" value="NONE"></el-option>
          </el-select>
        </el-form-item>
        <el-form-item label="Email Username">
          <el-input v-model="email['email.username']"></el-input>
        </el-form-item>
        <el-form-item label="Email Password">
          <el-input v-model="email['email.password']"></el-input>
        </el-form-item>
        <el-form-item label="Email From">
          <el-input v-model="email['email.from']"></el-input>
        </el-form-item>
        <el-button type="primary" @click="updateEmail">Update Email</el-button>
      </el-form-item>
    </el-form>
  </div>
</template>

<script lang="ts">
import {BasicResponse, SettingReport,} from "@/backend/Response";
import http from "@/http-common";
import {defineComponent, ref} from "vue";
import {useCookie} from "vue-cookie-next";

export default defineComponent({
  setup() {
    let settingForm = ref({
      "name.public": "Nitro Repo",
    });
    let email = ref({
      "email.host": "",
      "email.username": "",
      "email.password": "",
      "email.encryption": "",
      "email.from": "",
      "email.port": "",
    });
    let unchangeable = ref({
      installed: false,
      version: "0.1.0",
    });
    const cookie = useCookie();

    const loadSettings = async () => {
      try {
        const value = await http.get("/api/settings/report", {
          headers: {
            Authorization: "Bearer " + cookie.getCookie("token"),
          },
        });
        if (value.status != 200) {
          return [];
        }
        const data = value.data as BasicResponse<unknown>;
        if (data.success) {
          let report = data.data as SettingReport;
          settingForm.value["name.public"] = report.general.name.value;
          email.value["email.host"] = report.email.email_host.value;
          email.value["email.username"] = report.email.email_username.value;
          email.value["email.from"] = report.email.from.value;
          email.value["email.password"] = "";
          email.value["email.encryption"] = report.email.encryption.value;
          email.value["email.port"] = report.email.port.value;
          unchangeable.value.installed =
            report.general.installed.value == "true";
          unchangeable.value.version = report.general.version.value;
        }
      } catch (e) {
        console.log(e);
      }
    };
    loadSettings();
    const tab = ref(0);

    return { settingForm, email, tab, unchangeable };
  },
  methods: {
    async onSettingSubmit() {
      for (const [k, v] of Object.entries(this.settingForm)) {
        let update = {
          value: v,
        };
        let body = JSON.stringify(update);
        console.log(body);
        const res = await http.post(
          "/api/admin/setting/" + k + "/update",
          body,
          {
            headers: {
              "Content-Type": "application/json",
              Authorization: "Bearer " + this.$cookie.getCookie("token"),
            },
          }
        );
        if (res.status != 200) {
          console.log("Data" + res.data);
          return;
        }
        const result = res.data;
        let value = JSON.stringify(result);
        console.log(value);

        let response: BasicResponse<unknown> = JSON.parse(value);

        if (response.success) {
          this.$notify({
            title: "Updated Setting " + k,
            type: "success",
          });
        } else {
          this.$notify({
            title: "Unable to update Setting " + k,
            type: "error",
          });
        }
      }
    },
    async updateEmail() {
      for (const [k, v] of Object.entries(this.email)) {
        let update = {
          value: v,
        };
        let body = JSON.stringify(update);
        console.log(body);
        const res = await http.post(
          "/api/admin/setting/" + k + "/update",
          body,
          {
            headers: {
              "Content-Type": "application/json",
              Authorization: "Bearer " + this.$cookie.getCookie("token"),
            },
          }
        );
        if (res.status != 200) {
          console.log("Data" + res.data);
          return;
        }
        const result = res.data;
        let value = JSON.stringify(result);
        console.log(value);

        let response: BasicResponse<unknown> = JSON.parse(value);

        if (response.success) {
          this.$notify({
            title: "Updated Setting " + k,
            type: "success",
          });
        } else {
          this.$notify({
            title: "Unable to update Setting " + k,
            type: "error",
          });
        }
      }
    },
  },
});
</script>
<style scoped></style>
