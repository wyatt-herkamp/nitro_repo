<template>
  <main class="search">
    <h1>Search</h1>
    <form
      class="searchForm"
      @submit.prevent="runSearch">
      <input
        v-model="input"
        class="searchInput"
        :class="{ invalid: parseError !== undefined }"
        placeholder="Search, or write a query: scope == dev.kingtux and version ~= 1.*"
        aria-label="Search" />
      <SubmitButton>Search</SubmitButton>
    </form>

    <p
      v-if="parseError"
      class="parseError">
      {{ parseError }}
    </p>

    <details class="help">
      <summary>Query syntax</summary>
      <p>
        A plain word searches project keys, names and descriptions. For anything more specific,
        write a query:
      </p>
      <ul>
        <li><code>scope == dev.kingtux</code> — exact, case-insensitive</li>
        <li>
          <code>name ~= *-api</code> — glob, where <code>*</code> and <code>?</code> are wildcards
        </li>
        <li><code>not release_type == Snapshot</code> — negation</li>
        <li>
          <code>created &gt; 2024-01-01</code> — only on <code>created</code> and
          <code>updated</code>
        </li>
        <li>
          <code>(scope == a or scope == b) and version ~= 1.*</code> — grouping; <code>and</code> is
          implied between two conditions
        </li>
      </ul>
      <p>
        Fields:
        <code
          v-for="field in fields"
          :key="field.name"
          class="field"
          >{{ field.name }}</code
        >
      </p>
    </details>

    <p v-if="searched && results.length === 0 && !parseError">Nothing matched.</p>

    <table
      v-if="results.length > 0"
      class="results">
      <thead>
        <tr>
          <th>Project</th>
          <th>Version</th>
          <th>Repository</th>
          <th>Updated</th>
        </tr>
      </thead>
      <tbody>
        <tr
          v-for="result in results"
          :key="result.version_id">
          <td>
            <RouterLink
              :to="{ name: 'ProjectPageView', params: { projectId: result.project_id } }"
              >{{ result.project_key }}</RouterLink
            >
            <span
              v-if="result.description"
              class="description"
              >{{ result.description }}</span
            >
          </td>
          <td>{{ result.version }}</td>
          <td>{{ result.storage }}/{{ result.repository }}</td>
          <td>{{ new Date(result.updated_at).toLocaleDateString() }}</td>
        </tr>
      </tbody>
    </table>
  </main>
</template>

<script setup lang="ts">
import SubmitButton from "@/components/form/SubmitButton.vue";
import http from "@/http";
import { onMounted, ref } from "vue";
import { RouterLink, useRoute, useRouter } from "vue-router";

interface SearchResult {
  project_id: string;
  version_id: string;
  repository: string;
  storage: string;
  project_key: string;
  name: string;
  scope?: string;
  description?: string;
  version: string;
  updated_at: string;
}
interface SearchField {
  name: string;
  orderable: boolean;
}

const route = useRoute();
const router = useRouter();
const input = ref((route.query.q as string) ?? "");
const results = ref<Array<SearchResult>>([]);
const fields = ref<Array<SearchField>>([]);
const parseError = ref<string | undefined>(undefined);
const searched = ref(false);

// An operator is what distinguishes a query from a search term, so the box accepts both rather
// than making people choose a mode first.
function looksLikeAQuery(value: string): boolean {
  return /(==|!=|~=|!~|>=|<=|>|<)/.test(value);
}

async function runSearch() {
  parseError.value = undefined;
  searched.value = true;
  const term = input.value.trim();
  const params = looksLikeAQuery(term) ? { query: term } : { text: term };
  // Keep the query in the URL so a search can be linked to and survives a refresh.
  router.replace({ query: term ? { q: term } : {} });

  await http
    .get<{ results: Array<SearchResult> }>("/api/search", { params })
    .then((response) => {
      results.value = response.data.results;
    })
    .catch((error) => {
      results.value = [];
      // The API reports where a query went wrong; showing that beats "search failed".
      parseError.value = error.response?.data?.error ?? "Search failed.";
    });
}

onMounted(async () => {
  await http
    .get<Array<SearchField>>("/api/search/fields")
    .then((response) => {
      fields.value = response.data;
    })
    .catch(() => {});
  if (input.value) {
    await runSearch();
  }
});
</script>

<style scoped lang="scss">
.search {
  padding: 1rem;
  max-width: 60rem;
  margin: 0 auto;
}
.searchForm {
  display: flex;
  gap: 0.5rem;
  align-items: center;
}
.searchInput {
  // `flex-basis: 0` plus `min-width: 0` — the box has to be free to take whatever the button does
  // not. It had `flex: 1` next to a `SubmitButton`, which is `width: 100%`; the button's basis was
  // therefore the full row, leaving zero free space to grow into, and the input collapsed to its
  // padding — about one character wide, with no room to see what was typed.
  flex: 1 1 0;
  min-width: 0;
  padding: 0.5rem;
  font-family: inherit;
  &.invalid {
    border-color: #ff6b6b;
  }
}
// The submit sits beside the field here rather than under it, so it is sized by its label.
.searchForm :deep(.submitButton) {
  flex: 0 0 auto;
  width: auto;
}
.parseError {
  color: #ff6b6b;
}
.help {
  margin: 1rem 0;
  code {
    padding: 0 0.2rem;
  }
  .field {
    margin-right: 0.4rem;
  }
}
.results {
  width: 100%;
  border-collapse: collapse;
  th,
  td {
    text-align: left;
    padding: 0.4rem 0.6rem;
  }
  th {
    border-bottom: 1px solid currentColor;
  }
  .description {
    display: block;
    opacity: 0.75;
    font-size: 0.9rem;
  }
}
</style>
