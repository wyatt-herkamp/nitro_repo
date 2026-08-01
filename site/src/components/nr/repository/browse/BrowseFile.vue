<template>
  <a
    class="browseItem"
    data-type="file"
    :href="repositoryURL"
    target="_blank"
    rel="noopener"
    :title="`${icon.label} \u2014 ${formatFileSize(file.file_size)}`">
    <span class="itemAndName">
      <font-awesome-icon
        :icon="icon.icon"
        class="itemIcon"
        :style="{ color: `var(${icon.color})` }" />
      <span class="itemName">{{ file.name }}</span>
    </span>

    <span class="itemMeta itemSize">{{ formatFileSize(file.file_size) }}</span>
    <span class="itemMeta itemModified">{{ formatDate(file.modified) }}</span>
  </a>
</template>

<script setup lang="ts">
/**
 * A file row in the repository browser.
 *
 * An `<a>` rather than a `div` with a click handler, so middle-click, "open in new tab" and the
 * status-bar URL preview all work — this row's whole purpose is to lead to a downloadable artifact.
 *
 * `mime_type` and `file_size` were already on every entry the backend sent and neither was ever
 * displayed. (#497)
 */
import { fixCurrentPath, type RawFile } from "@/types/browse";
import { createRepositoryRoute, type RepositoryWithStorageName } from "@/types/repository";
import { formatDate, formatFileSize } from "@/utils/format";
import { computed, type PropType } from "vue";
import { iconForFile } from "./fileIcons";
import "./browse.scss";

const props = defineProps({
  file: {
    type: Object as PropType<RawFile>,
    required: true,
  },
  currentPath: {
    type: String,
    required: true,
  },
  repository: {
    type: Object as PropType<RepositoryWithStorageName>,
    required: true,
  },
});

const fixedPath = fixCurrentPath(props.currentPath);
const repositoryURL = createRepositoryRoute(props.repository, `${fixedPath}/${props.file.name}`);

const icon = computed(() => iconForFile(props.file.name, props.file.mime_type));
</script>
