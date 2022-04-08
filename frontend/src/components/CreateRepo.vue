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
          bg-slate-800
          border-black
          m-w-20
          py-5
          px-10
          rounded-2xl
          shadow-xl
          text-center
        "
      >
        <p class="font-bold text-xl pb-4">Create Repository</p>
        <form class="flex flex-col w-96 <sm:w-65" @submit.prevent="onSubmit()">
          <div class="mb-4">
            <label
              class="block text-slate-50 text-sm font-bold mb-2"
              for="name"
            >
              Repository Name
            </label>
            <input
              id="name"
              v-model="form.name"
              autocomplete="off"
              required
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
              placeholder="Repository Name"
              type="text"
            />
          </div>
          <div class="flex flex-row">
            <div class="grow pr-2">
              <label
                class="block text-slate-50 text-sm font-bold mb-2"
                for="name"
              >
                Repository Type
              </label>
              <select
                id="type"
                v-model="form.type"
                required
                class="
                  border border-gray-300
                  rounded
                  text-gray-600
                  h-10
                  px-5
                  w-full
                  bg-white
                  hover:border-gray-400
                  focus:outline-none
                  appearance-none
                "
              >
                <option disabled selected value="">Repository Type</option>

                <option value="maven">Maven</option>
                <option value="npm">NPM</option>
              </select>
            </div>
          </div>
          <button
            class="bg-slate-900 py-2 my-3 rounded-md cursor-pointer text-white"
          >
            Create Repository
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
import { Repository } from "nitro_repo-api-wrapper";
import { defineComponent, ref } from "vue";
import { useCookie } from "vue-cookie-next";
import { getStorages } from "nitro_repo-api-wrapper";
import { createNewRepository } from "nitro_repo-api-wrapper";

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
    const cookie = useCookie();
    const isLoading = ref(false);
    const showModel = ref(false);

    const error = ref("");
    const getStorage = async () => {
      isLoading.value = true;
      try {
        const value = await getStorages(cookie.getCookie("token"));
        storages.value = value;

        isLoading.value = false;
      } catch (e) {
        error.value = "Error";
      }
    };
    const close = () => (showModel.value = false);

    getStorage();
    return {
      form,
      isLoading,
      error,
      getStorage,
      showModel,
      close,
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
});
</script>
<style scoped></style>
