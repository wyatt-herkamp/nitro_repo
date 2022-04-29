<template>
  <div class="flex w-full">
    <form class="appearance-none flex-col" @submit.prevent="onSubmit()">
      <div class="flex flex-row">
        <div class="grow">
          <p class="header">Create Storage</p>
        </div>
        <div class="m-auto pt-5 pr-3">
          <button
            type="button"
            class="xButton block"
            @click="showModel = false"
          >
            🗙
          </button>
        </div>
      </div>

      <div class="flex-row">
        <div class="px-3">
          <label class="nitroLabel" for="name"> Storage ID/Name </label>
          <input
            id="name"
            v-model="form.name"
            autocomplete="off"
            class="nitroTextInput"
            placeholder="Storage ID/Name"
            type="text"
          />
        </div>
      </div>
      <div class="flex-row">
        <div class="px-3">
          <label class="nitroLabel" for="name"> Storage Public Name </label>
          <input
            id="name"
            v-model="form.public_name"
            autocomplete="off"
            class="nitroTextInput"
            placeholder="Public Name"
            type="text"
          />
        </div>
      </div>
      <button class="nitroButtonLight">Create Storage</button>
    </form>
  </div>
</template>

<script lang="ts">
import { Storage } from "@nitro_repo/nitro_repo-api-wrapper";
import { defineComponent, inject, ref, watch } from "vue";
import { createNewStorage } from "@nitro_repo/nitro_repo-api-wrapper";
import { useRouter } from "vue-router";

export default defineComponent({
  props: {
    modelValue: Boolean,
  },
  setup(props, { emit }) {
    const showModel = ref(props.modelValue);
    const token: string | undefined = inject("token");

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
    let form = ref({
      name: "",
      public_name: "",
      error: "",
    });
    return { form, showModel, close, token: token as string };
  },
  methods: {
    async onSubmit() {
      const response = await createNewStorage(
        this.form.name,
        this.form.public_name,
        this.token
      );
      if (response.ok) {
        let data = response.val as Storage;
        this.$notify({
          title: "Storage Created",
          type: "success",
        });
        this.$router.push("/admin/storage/" + data.name);
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
