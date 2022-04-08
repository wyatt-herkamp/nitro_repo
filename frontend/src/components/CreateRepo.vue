<template>
  <NitroModal v-model="showModel">
    <template v-slot:header> Create Repository </template>
    <template v-slot:content>
      <form class="flex flex-col w-96 <sm:w-65" @submit.prevent="onSubmit()">
        <div class="mb-4">
          <label class="nitroLabel" for="name"> Repository Name </label>
          <input
            id="name"
            v-model="form.name"
            autocomplete="off"
            required
            class="nitroTextInput"
            placeholder="Repository Name"
            type="text"
          />
        </div>
        <div class="flex flex-row">
          <div class="grow pr-2">
            <label class="nitroLabel" for="name"> Repository Type </label>
            <select
              id="type"
              v-model="form.type"
              required
              class="nitroSelectBox"
            >
              <option disabled selected value="">Repository Type</option>
              <option value="maven">Maven</option>
              <option value="npm">NPM</option>
            </select>
          </div>
        </div>
        <button class="nitroButtonLight">Create Repository</button>
      </form>
    </template>
    <template v-slot:button>
      <button class="openModalButton">Create Repository</button>
    </template>
  </NitroModal>
</template>
<style scoped>
</style>
<script lang="ts">
import { Repository } from "nitro_repo-api-wrapper";
import { defineComponent, ref } from "vue";
import { createNewRepository } from "nitro_repo-api-wrapper";
import NitroModal from "./common/model/NitroModal.vue";

export default defineComponent({
  props: {
    storage: {
      type: Object as () => Storage,
      required: true,
    },
  },
  setup() {
    let form = ref({
      name: "",
      type: "",
      error: "",
    });
    const showModel = ref(false);

    return {
      form,
      showModel,
    };
  },
  methods: {
    async onSubmit() {
      if (this.form.type === "") {
        this.$notify({
          title: "Please Specify a Repository Type",
          type: "warn",
        });
        return;
      }
      const response = await createNewRepository(
        this.form.name,
        this.$props.storage.name,
        this.form.type,
        this.$cookie.getCookie("token")
      );
      if (response.ok) {
        let data = response.val as Repository;
        this.$notify({
          title: "Repository Created",
          type: "success",
        });
        this.$router.push(
          "/admin/repository/" + data.storage + "/" + data.name
        );
      } else {
        this.$notify({
          title: "Unable to Create Repository",
          text: JSON.stringify(response.val.user_friendly_message),
          type: "error",
        });
      }
    },
  },
  components: { NitroModal },
});
</script>
<style scoped></style>
