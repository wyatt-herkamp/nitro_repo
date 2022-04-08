<template>
  <NitroModal v-model="showModel">
    <template v-slot:header>
      Delete {{ repository.storage }}/{{ repository.name }}
    </template>
    <template v-slot:content>
      <form class="flex flex-col w-96 <sm:w-65" @submit.prevent="onSubmit()">
        <div class="mb-4">
          <Switch id="deleteFiles" v-model="deleteFiles">
            <div class="ml-3 text-slate-50 font-medium">Delete Files</div>
          </Switch>
        </div>
        <button
          class="
            bg-slate-900
            py-2
            my-3
            hover:bg-red-700
            rounded-md
            cursor-pointer
            text-white
          "
        >
          Delete Repository
        </button>
      </form>
    </template>
    <template v-slot:button>
      <button class="nitroButton">Delete Repository</button>
    </template>
  </NitroModal>
</template>


<script lang="ts">
import { Repository } from "nitro_repo-api-wrapper";
import { defineComponent, ref } from "vue";
import { useCookie } from "vue-cookie-next";
import { deleteRepository } from "nitro_repo-api-wrapper";
import NitroModal from "@/components/common/model/NitroModal.vue";

export default defineComponent({
  props: {
    repository: {
      type: Object as () => Repository,
      required: true,
    },
  },
  setup() {
    let deleteFiles = ref(false);
    const cookie = useCookie();
    const isLoading = ref(false);
    const showModel = ref(false);
    const error = ref("");
    const close = () => (showModel.value = false);
    return {
      deleteFiles,
      isLoading,
      error,
      showModel,
      close,
    };
  },
  methods: {
    async onSubmit() {
      const response = await deleteRepository(
        this.$props.repository.name,
        this.$props.repository.storage,
        this.deleteFiles,
        this.$cookie.getCookie("token")
      );
      if (response.ok) {
        let data = response.val as Repository;
        this.$notify({
          title: "Repository Deleted",
          type: "success",
        });
        this.$router.push("/admin/storage/" + this.$props.repository.storage);
      } else {
        this.$notify({
          title: "Unable to Delete Repository",
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
