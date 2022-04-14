<template>
  <div class="flex w-full">
    <form class="nitroForm" @submit.prevent="onSubmit()">
      <div class="flex flex-row">
        <div class="grow">
          <p class="header">Create Repository</p>
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

      <div class="settingRow">
        <div class="settingBox">
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
      </div>
      <div class="settingRow">
        <div class="settingBox">
          <label class="nitroLabel" for="name"> Repository Type </label>
          <select id="type" v-model="form.type" required class="nitroSelectBox">
            <option disabled selected value="">Repository Type</option>
            <option value="Maven">Maven</option>
            <option value="NPM">NPM</option>
          </select>
        </div>
      </div>
      <button class="nitroButtonLight">Create Repository</button>
    </form>
  </div>
</template>
<style scoped></style>
<script lang="ts">
import { Repository } from "nitro_repo-api-wrapper";
import { defineComponent, inject, ref, watch } from "vue";
import { createNewRepository } from "nitro_repo-api-wrapper";
import NitroModal from "@/components/common/model/NitroModal.vue";
import { useRouter } from "vue-router";

export default defineComponent({
  props: {
    storage: {
      type: Object as () => Storage,
      required: true,
    },
    modelValue: Boolean,
  },
  setup(props, { emit }) {
    const token: string | undefined = inject("token");
    if (token == undefined) {
      useRouter().push("login");
    }
    let form = ref({
      name: "",
      type: "",
      error: "",
    });
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
    return {
      form,
      showModel,
      token: token as string,
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
        this.token
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
