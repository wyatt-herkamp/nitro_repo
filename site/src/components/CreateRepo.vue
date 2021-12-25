<template>

</template>

<script lang="ts">
import {DEFAULT_STORAGE_LIST, Repository,} from "@/backend/Response";
import {defineComponent, ref} from "vue";
import {useCookie} from "vue-cookie-next";
import {getStorages} from "@/backend/api/Storages";
import {createNewRepository} from "@/backend/api/admin/Repository";

export default defineComponent({
  props: {
    updateList: {
      required: true,
      type: Function,
    },
  },
  setup() {
    let form = ref({
      name: "",
      storage: "",
      type: "",
      error: "",
    });
    const cookie = useCookie();
    const isLoading = ref(false);

    const error = ref("");
    let storages = ref(DEFAULT_STORAGE_LIST);
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
    getStorage();
    return {
      form,
      storages,
      isLoading,
      error,
      getStorage,
    };
  },
  methods: {
    async onSubmit() {
      const response = await createNewRepository(
        this.form.name,
        this.form.storage,
        this.form.type,
        this.$cookie.getCookie("token")
      );
      if (response.ok) {
        let data = response.val as Repository;
        this.$props.updateList(data.id);
        this.$notify({
          title: "Repository Created",
          type: "success",
        });
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
