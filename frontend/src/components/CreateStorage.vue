<template>
  <div>
    <form @submit.prevent="onSubmit()">
      <div class="sideHeader">
        <p class="headerOne">Create Storage</p>
      </div>
      <div class="flex-row">
        <div class="px-3">
          <label class="nitroLabel" for="name"> Storage ID </label>
          <input
            id="name"
            v-model="form.id"
            autocomplete="off"
            class="nitroTextInput"
            placeholder="Public Name"
            type="text"
          />
        </div>
      </div>
      <div class="flex-row">
        <div class="px-3">
          <label class="nitroLabel" for="storageType"> Storage Type </label>
          <select
            id="storageType"
            v-model="form.type"
            required
            class="nitroSelectBox"
          >
            <option disabled selected value="">Storage Type</option>
            <option value="LocalStorage">Local Storage</option>
          </select>
        </div>
      </div>
      <div class="border-t-2 mt-2">
        <div v-if="form.type === 'LocalStorage'">
          <div class="flex-row">
            <div class="px-3">
              <label class="nitroLabel" for="storagePath"> Storage Path </label>
              <input
                id="storagePath"
                v-model="localStorageConf.location"
                autocomplete="off"
                class="nitroTextInput"
                placeholder="Storage Path"
                type="text"
              />
              <h6 class="text-quaternary pl-2 text-xs">
                {local_storage_folder} and {storage_id} are variables to the
                system
              </h6>
            </div>
          </div>
        </div>
      </div>
      <div class="flex flex-row h-12 mt-5">
        <button :disabled="form.id.length === 0" class="buttonOne">
          Create Storage
        </button>
      </div>
    </form>
  </div>
</template>

<script lang="ts">
import { computed, defineComponent, ref, watch } from "vue";
import "@/styles/sideCreate.css";
import httpCommon from "@/http-common";
import { Storage } from "@/types/storageTypes";
export default defineComponent({
  props: {
    modelValue: Boolean,
    storagesThatExist: {
      type: Object as () => Storage[],
      required: true,
    },
  },
  setup(props, { emit }) {
    const showModel = ref(props.modelValue);
    watch(
      () => props.modelValue,
      (val) => {
        showModel.value = val;
        emit("update:modelValue", val);
      }
    );
    watch(showModel, (val) => {
      emit("update:modelValue", val);
    });
    const form = ref({ id: "", type: "LocalStorage" });
    const localStorageConf = ref({
      location: "{local_storage_folder}/{storage_id}",
    });
    const validId = computed(() => {
      if (form.value.id.length == 0) {
        return false;
      }
      for (const storage of props.storagesThatExist) {
        if (storage.id == form.value.id) {
          return false;
        }
      }
      return true;
    });
    return { form, showModel, close, validId, localStorageConf };
  },
  methods: {
    async onSubmit() {
      if (!this.validId) {
        this.$notify({
          type: "warn",
          title: "The storage ID is already in use",
        });
        return;
      }

      await httpCommon.apiClient
        .post("api/admin/storage/new", {
          id: this.form.id,
          handler_config: {
            [this.form.type]: this.localStorageConf,
          },
        })
        .then((res) => {
          if (res.status == 200) {
            this.$notify({
              title: "Created Storage",
              type: "success",
            });
            this.$router.push(`/admin/storage/${this.form.id}`);
          } else if (res.status == 409) {
            this.$notify({
              title: "Storage Already Exists",
              type: "error",
            });
          } else {
            this.$notify({
              title: "Error Creating Storage",
              text: res.data,
              type: "error",
            });
          }
        });
    },
  },
});
</script>
<style scoped></style>
