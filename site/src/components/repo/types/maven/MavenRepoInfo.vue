<template>
  <pre v-highlightjs>
      <code class="xml">&lt;repository&gt;
    &lt;id&gt;{{repository.name}}&lt;/id&gt;
    &lt;url&gt;{{url}}/{{repository.storage}}/{{repository.name}}&lt;/url&gt;
&lt;/repository&gt;
</code>
</pre>
</template>
<style scoped></style>
<script lang="ts">
import { Repository } from "@/backend/Response";
import { apiURL } from "@/http-common";
import { defineComponent, ref } from "vue";

export default defineComponent({
  props: {
    repository: {
      required: true,
      type: Object as () => Repository,
    },
  },
  setup() {
    let url = apiURL;
    return { url };
  },
});
</script>
