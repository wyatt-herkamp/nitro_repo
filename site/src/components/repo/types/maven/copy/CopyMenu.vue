<template>
  <nav class="flex flex-wrap bg-slate-800 p-6 m-1 roun"></nav>

  <pre v-highlightjs>
      <code class="xml">&lt;repository&gt;
    &lt;id&gt;{{repository.name}}&lt;/id&gt;
    &lt;url&gt;{{url}}/{{repository.storage}}/{{repository.name}}&lt;/url&gt;
&lt;/repository&gt;
</code>
</pre>
</template>

<script lang="ts">
import { defineComponent, ref } from "vue";
import { useRouter } from "vue-router";
import CopyItem from "@/components/repo/types/maven/copy/CopyItem.vue";
import { Repository } from "@/backend/Response";
import { apiURL } from "@/http-common";
import { PublicRepositoryInfo } from "@/backend/api/Repository";

export default defineComponent({
  components: { CopyItem },
  props: {
    repository: {
      required: true,
      type: Object as () => Repository | PublicRepositoryInfo,
    },
  },
  setup() {
    const url = apiURL;
    return { url };
  },
  methods: {
    changeViewValue(value: string) {
      console.log(value);
      this.$emit("changeView", value);
    },
  },
});
</script>