<template>
  <div class="settingContent">
    <h2 class="settingHeader">Repository Badge Settings</h2>
    <div class="border-b-2 border-slate-900">
      <div class="settingSection">
        <div class="settingBox">
          <label for="grid-label-color">Label Color </label>
          <div :style="{ background: repository.settings.badge.label_color }">
            <ColorPicker
              theme="dark"
              :color="repository.settings.badge.label_color"
              :sucker-hide="false"
              :sucker-canvas="labelSuckerCanvas"
              :sucker-area="labelSuckerArea"
              @changeColor="changeLabelColor"
            />
          </div>
        </div>
        <div class="settingBox">
          <label for="grid-color">Color</label>
          <div :style="{ background: repository.settings.badge.color }">
            <ColorPicker
              theme="dark"
              :color="repository.settings.badge.color"
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
          <label for="grid-style">Badge Style</label>
          <select
            v-model="repository.settings.badge.style"
            class="nitroTextInput"
          >
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

    <h2 class="settingHeader">Repository Page Settings</h2>
    <div class="settingSection">
      <div class="settingBox">
        <label for="grid-policy">Page Provider</label>
        <select
          v-model="repository.settings.frontend.page_provider"
          class="nitroTextInput"
        >
          <option>None</option>
          <option value="ReadmeSent">README Sent</option>
          <option value="ReadmeGit">README Git</option>
        </select>
      </div>

      <div class="settingBox">
        <label for="grid-active">Frontend Page Enabled</label>
        <select
          v-model="repository.settings.frontend.enabled"
          class="nitroTextInput"
        >
          <option>true</option>
          <option>false</option>
        </select>
      </div>
      <div class="settingBox">
        <button class="nitroButton" @click="submitFrontend()">
          Update Frontend
        </button>
      </div>
    </div>
  </div>
</template>
<script lang="ts">
import { defineComponent, inject } from "vue";
import { Repository } from "@/types/repositoryTypes";

import { ColorPicker } from "vue-color-kit";
import "vue-color-kit/dist/vue-color-kit.css";

export default defineComponent({
  components: {
    ColorPicker,
  },
  props: {
    repository: {
      required: true,
      type: Object as () => Repository,
    },
  },
  data() {
    const token = inject("token") as string;

    return {
      token: token,
      //Honestly I really dont know. I am just going to let it do its thing
      labelSuckerCanvas: null,
      labelSuckerArea: [],
      labelIsSucking: false,
      badgeSuckerCanvas: null,
      badgeSuckerArea: [],
      badgeIsSucking: false,
    };
  },
  methods: {
    changeLabelColor(color: { hex: string }) {
      this.repository.settings.badge.label_color = color.hex;
    },
    changeBadgeColor(color: { hex: string }) {
      this.repository.settings.badge.color = color.hex;
    },
    async submitBadge() {
      // TODO submit badge settings
    },
    async submitFrontend() {
      // TODO submit frontend settings
    },
  },
});
</script>
