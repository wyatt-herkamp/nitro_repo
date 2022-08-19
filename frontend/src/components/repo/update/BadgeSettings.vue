<template>
  <div class="settingContent">
    <h2 class="settingHeader">Badge Settings</h2>
    <div class="settingSection">
      <div class="settingBox">
        <label for="grid-label-color" class="nitroLabel">Label Color </label>
        <div :style="{ background: badgeSettings.label_color }">
          <ColorPicker
            v-if="badgeSettings.style !== ''"
            theme="dark"
            :color="badgeSettings.label_color"
            :sucker-hide="false"
            :sucker-canvas="labelSuckerCanvas"
            :sucker-area="labelSuckerArea"
            @changeColor="changeLabelColor"
          />
        </div>
      </div>
      <div class="settingBox">
        <label for="grid-color" class="nitroLabel"
          >Color: {{ badgeSettings.color }}</label
        >
        <div :style="{ background: badgeSettings.color }">
          <ColorPicker
            theme="dark"
            v-if="badgeSettings.style !== ''"
            :color="badgeSettings.color"
            :sucker-hide="false"
            :sucker-canvas="badgeSuckerCanvas"
            :sucker-area="badgeSuckerArea"
            @changeColor="changeBadgeColor"
          />
        </div>
      </div>
    </div>
    <div class="settingSection">
      <div class="settingBox">
        <label for="grid-style" class="nitroLabel">Badge Style</label>
        <select v-model="badgeSettings.style" class="nitroTextInput">
          <option value="Flat">Flat</option>
          <option value="FlatSquare">Flat Square</option>
          <option value="Plastic">Platic</option>
        </select>
      </div>
      <div class="settingBox">
        <button class="nitroButton" @click="submitBadge()">
          Update Badge Settings
        </button>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import { computed, defineComponent, ref } from "vue";
import httpCommon from "@/http-common";
import { ColorPicker } from "vue-color-kit";
import "vue-color-kit/dist/vue-color-kit.css";
import { notify } from "@kyvg/vue3-notification";

export default defineComponent({
  name: "BadgeSettings",
  components: { ColorPicker },
  data() {
    return {
      //Honestly I really dont know. I am just going to let it do its thing
      labelSuckerCanvas: null,
      labelSuckerArea: [],
      labelIsSucking: false,
      badgeSuckerCanvas: null,
      badgeSuckerArea: [],
      badgeIsSucking: false,
    };
  },
  props: {
    repository: {
      type: Object as () => { storage: string; name: string },
      required: true,
    },
  },
  setup(props) {
    const badgeSettings = ref({
      color: "",
      label_color: "",
      style: "",
    });

    httpCommon.apiClient
      .get(
        `api/admin/repositories/${props.repository.storage}/${props.repository.name}/config/badge`
      )
      .then((response) => {
        badgeSettings.value = response.data;
        console.log(badgeSettings.value);
      });
    return { badgeSettings };
  },
  methods: {
    changeLabelColor(color: { hex: string }) {
      this.badgeSettings.label_color = color.hex;
    },
    changeBadgeColor(color: { hex: string }) {
      this.badgeSettings.color = color.hex;
    },
    async submitBadge() {
      await httpCommon.apiClient
        .put(
          `api/admin/repositories/${this.repository.storage}/${this.repository.name}/config/badge`,
          this.badgeSettings
        )
        .then((response) => {
          if (response.status === 204) {
            this.$notify({
              title: "Success",
              type: "success",
            });
          } else {
            this.$notify({
              title: "Error",
              type: "error",
            });
            console.log(response);
          }
        })
        .catch((error) => {
          this.$notify({
            title: "Error",
            type: "error",
          });
          console.log(error);
        });
    },
  },
});
</script>

<style scoped></style>
