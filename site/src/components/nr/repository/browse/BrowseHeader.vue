<template>
  <NBreadcrumb :items="crumbs" />
</template>

<script setup lang="ts">
/**
 * The path trail above the browser.
 *
 * `buildPath` used to be an unguarded `(route.params.catchAll as string).split("/")`, which throws
 * on a root browse — where there is no `catchAll` at all — taking the page down with it. It also
 * pushed a separate `{ name: "/" }` entry between every segment, making the separator a list item
 * rather than presentation, and logged the whole trail on every navigation.
 */
import { computed } from "vue";
import { useRoute } from "vue-router";
import NBreadcrumb, { type Crumb } from "@/components/core/ui/NBreadcrumb.vue";
import type { RepositoryWithStorageName } from "@/types/repository";

const props = defineProps<{ repository: RepositoryWithStorageName }>();

const route = useRoute();

const crumbs = computed<Array<Crumb>>(() => {
  const items: Array<Crumb> = [
    {
      label: `${props.repository.storage_name}/${props.repository.name}`,
      to: `/browse/${props.repository.id}`,
    },
  ];

  const catchAll = route.params.catchAll;
  const path = Array.isArray(catchAll) ? catchAll.join("/") : (catchAll ?? "");

  let accumulated = "";
  for (const segment of path.split("/").filter(Boolean)) {
    accumulated += `${segment}/`;
    items.push({
      label: segment,
      to: `/browse/${props.repository.id}/${accumulated}`,
    });
  }

  return items;
});
</script>
