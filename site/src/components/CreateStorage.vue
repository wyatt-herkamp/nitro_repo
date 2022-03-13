<template>
  <div>
    <vue-final-modal
      v-model="showModel"
      classes="flex justify-center items-center"
    >
      <div
        class="
          relative
          border
          bg-slate-900
          border-black
          m-w-20
          py-5
          px-10
          rounded-2xl
          shadow-xl
          text-center
        "
      >
        <p class="font-bold text-xl pb-4">Create Storage</p>
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
              class="
                shadow
                appearance-none
                border
                rounded
                w-full
                py-2
                px-3
                text-gray-700
                leading-tight
                focus:outline-none focus:shadow-outline
              "
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
              class="
                shadow
                appearance-none
                border
                rounded
                w-full
                py-2
                px-3
                text-gray-700
                leading-tight
                focus:outline-none focus:shadow-outline
              "
              placeholder="Public Name"
              type="text"
            />
          </div>
          <button
            class="
             bg-slate-800
              py-2
              my-3
              rounded-md
              cursor-pointer
              text-white
            "
          >
            Create Storage
          </button>
        </form>

        <button class="absolute top-0 right-0 mt-5 mr-5" @click="close()">
          🗙
        </button>
      </div>
    </vue-final-modal>
    <div @click="showModel = true">
      <slot name="button"></slot>
    </div>
  </div>
</template>

<script lang="ts">
import { Storage } from "@/backend/Response";
import { defineComponent, ref } from "vue";
import { createNewStorage } from "@/backend/api/admin/Storage";

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
