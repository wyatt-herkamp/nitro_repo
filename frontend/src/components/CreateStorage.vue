
<template>
  <NitroModal v-model="showModel">
    <template v-slot:header> Create Storage </template>
    <template v-slot:content>
      <form class="flex flex-col w-96 <sm:w-65" @submit.prevent="onSubmit()">
        <form class="flex flex-col w-96 <sm:w-65" @submit.prevent="onSubmit()">
          <div class="mb-4">
            <label
              class="block text-slate-50 text-sm font-bold mb-2"
              for="name"
            >
              Storage ID/Name
            </label>
            <input
              id="name"
              v-model="form.name"
              autocomplete="off"
              class="nitroTextInput"
              placeholder="Storage ID/Name"
              type="text"
            />
          </div>
          <div class="mb-4">
            <label
              class="block text-slate-50 text-sm font-bold mb-2"
              for="name"
            >
              Storage Public Name
            </label>
            <input
              id="name"
              v-model="form.public_name"
              autocomplete="off"
              class="nitroTextInput"
              placeholder="Public Name"
              type="text"
            />
          </div>
          <button class="nitroButtonLight">Create Storage</button>
        </form>
      </form>
    </template>
    <template v-slot:button>
      <button class="openModalButton">Create Storage</button>
    </template>
  </NitroModal>
</template>

<script lang="ts">
import { Storage } from "nitro_repo-api-wrapper";
import { defineComponent, ref } from "vue";
import { createNewStorage } from "nitro_repo-api-wrapper";

export default defineComponent({
  setup() {
    const showModel = ref(false);
    const close = () => (showModel.value = false);

    let form = ref({
      name: "",
      public_name: "",
      error: "",
    });
    return { form, showModel, close };
  },
  methods: {
    async onSubmit() {
      const response = await createNewStorage(
        this.form.name,
        this.form.public_name,
        this.$cookie.getCookie("token")
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
