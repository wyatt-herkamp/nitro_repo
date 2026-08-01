<template>
  <div class="stack">
    <NCard
      title="Badge"
      subtitle="The SVG served at the badge endpoints for this repository and its projects.">
      <div class="badgeGrid">
        <div class="controls">
          <div class="field">
            <label :for="`${id}-style`">Style</label>
            <select
              :id="`${id}-style`"
              v-model="config.badge_settings.style">
              <option value="flat">Flat</option>
              <option value="plastic">Plastic</option>
              <option value="flatsquare">Flat square</option>
            </select>
          </div>

          <div class="field">
            <label :for="`${id}-label-color`">Label colour</label>
            <div class="colorRow">
              <input
                :id="`${id}-label-color`"
                v-model="config.badge_settings.label_color"
                type="color" />
              <input
                v-model="config.badge_settings.label_color"
                class="mono"
                aria-label="Label colour hex"
                spellcheck="false" />
            </div>
          </div>

          <div class="field">
            <label :for="`${id}-color`">Value colour</label>
            <div class="colorRow">
              <input
                :id="`${id}-color`"
                v-model="config.badge_settings.color"
                type="color" />
              <input
                v-model="config.badge_settings.color"
                class="mono"
                aria-label="Value colour hex"
                spellcheck="false" />
            </div>
          </div>

          <div class="field">
            <label class="checkboxLabel">
              <input
                v-model="config.require_semver"
                type="checkbox" />
              Require semver versions
            </label>
            <span class="field-hint">
              Rejects a deploy whose version is not valid semver. Unrelated to badges; it is the
              other half of this repository's project config.
            </span>
          </div>
        </div>

        <div class="preview">
          <span class="label">Preview</span>
          <!-- Rendered locally rather than by fetching the endpoint, so the preview follows the
               colour picker live instead of only after a save. -->
          <div
            class="previewBadge"
            v-html="previewSvg" />
          <span class="field-hint"
            >Saved settings apply to every badge this repository serves.</span
          >
        </div>
      </div>

      <template #actions>
        <NButton
          variant="primary"
          :loading="saving"
          @click="save">
          Save
        </NButton>
      </template>
    </NCard>

    <NCard
      title="Usage"
      subtitle="Paste one of these where the badge should appear.">
      <CodeMenu
        :snippets="snippets"
        default-tab="markdown" />
    </NCard>
  </div>
</template>

<script setup lang="ts">
/**
 * Badge settings. (#501)
 *
 * There were zero occurrences of "badge" anywhere in `site/`, despite the backend serving the SVGs
 * and persisting `ProjectConfig.badge_settings` with a schema. Because `"project"` was missing from
 * `configTypes`, these settings rendered through the generic JSON-schema fallback: raw hex text
 * inputs, no preview, and no indication of what the endpoint URL even was.
 */
import { computed, ref, useId } from "vue";
import { notify } from "@kyvg/vue3-notification";
import NCard from "@/components/core/ui/NCard.vue";
import NButton from "@/components/core/ui/NButton.vue";
import CodeMenu from "@/components/core/code/CodeMenu.vue";
import http from "@/http";
import { apiURL } from "@/config";
import { useRepositoryStore } from "@/stores/repositories";
import type { CodeSnippet } from "@/components/core/code/code";
import type { RepositoryWithStorageName } from "@/types/repository";

// The config host passes the repository id, matching every other config component.
const props = defineProps<{ repository: string }>();

interface BadgeSettings {
  style: string;
  label_color: string;
  color: string;
}
interface ProjectConfig {
  badge_settings: BadgeSettings;
  require_semver: boolean;
}

const id = useId();
const saving = ref(false);
const repositoryStore = useRepositoryStore();
const repositoryDetails = ref<RepositoryWithStorageName | undefined>(undefined);

const config = ref<ProjectConfig>({
  badge_settings: { style: "flat", label_color: "#555555", color: "#33B5E5" },
  require_semver: false,
});

async function load() {
  repositoryDetails.value = await repositoryStore.getRepositoryById(props.repository, false);
  try {
    // `?default=true` so a repository that has never saved this config still renders the form the
    // server would accept, rather than an empty one.
    const response = await http.get<ProjectConfig>(
      `/api/repository/${props.repository}/config/project?default=true`,
    );
    if (response.data) config.value = response.data;
  } catch {
    notify({ type: "error", title: "Could not load badge settings" });
  }
}
load();

const badgeUrl = computed(() => {
  const base = apiURL.endsWith("/") ? apiURL.slice(0, -1) : apiURL;
  const details = repositoryDetails.value;
  if (!details) return "";
  return `${base}/badge/${details.storage_name}/${details.name}`;
});

const snippets = computed<Array<CodeSnippet>>(() => {
  const url = badgeUrl.value;
  const name = repositoryDetails.value?.name ?? "repository";
  const target = `${window.location.origin}/browse/${props.repository}`;
  return [
    {
      name: "Markdown",
      key: "markdown",
      language: "markdown",
      code: `[![${name}](${url})](${target})`,
    },
    {
      name: "HTML",
      key: "html",
      language: "xml",
      code: `<a href="${target}"><img src="${url}" alt="${name}" /></a>`,
    },
    {
      name: "reStructuredText",
      key: "rst",
      language: "markdown",
      code: `.. image:: ${url}\n   :target: ${target}\n   :alt: ${name}`,
    },
    {
      name: "Project badge",
      key: "project",
      language: "markdown",
      code: `[![version](${url}/project/{PROJECT_KEY})](${target})`,
    },
  ];
});

/**
 * A local approximation of what `badge_maker` renders. Character widths in a proportional face
 * cannot be measured without laying the text out, so this estimates them — the preview is for
 * judging colour, and the served badge is the real thing.
 */
const previewSvg = computed(() => {
  const settings = config.value.badge_settings;
  const label = "Repository";
  const value = repositoryDetails.value?.name ?? "repository";

  const width = (text: string) => Math.round(text.length * 6.6) + 20;
  const labelWidth = width(label);
  const valueWidth = width(value);
  const total = labelWidth + valueWidth;
  const height = settings.style === "plastic" ? 18 : 20;
  const radius = settings.style === "flatsquare" ? 0 : 3;

  return `<svg xmlns="http://www.w3.org/2000/svg" width="${total}" height="${height}" role="img">
  <rect width="${total}" height="${height}" rx="${radius}" fill="${escapeAttribute(settings.label_color)}"/>
  <rect x="${labelWidth}" width="${valueWidth}" height="${height}" rx="${radius}" fill="${escapeAttribute(settings.color)}"/>
  <rect x="${labelWidth}" width="4" height="${height}" fill="${escapeAttribute(settings.color)}"/>
  <g fill="#fff" font-family="Verdana,Geneva,DejaVu Sans,sans-serif" font-size="11">
    <text x="${labelWidth / 2}" y="${height / 2 + 4}" text-anchor="middle">${escapeText(label)}</text>
    <text x="${labelWidth + valueWidth / 2}" y="${height / 2 + 4}" text-anchor="middle">${escapeText(value)}</text>
  </g>
</svg>`;
});

// The repository name reaches the preview through `v-html`, so it has to be escaped here.
function escapeText(value: string): string {
  return value.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}

function escapeAttribute(value: string): string {
  return escapeText(value).replace(/"/g, "&quot;");
}

async function save() {
  saving.value = true;
  try {
    await http.put(`/api/repository/${props.repository}/config/project`, config.value);
    notify({ type: "success", title: "Badge settings saved" });
  } catch {
    notify({ type: "error", title: "Could not save badge settings" });
  } finally {
    saving.value = false;
  }
}
</script>

<style scoped lang="scss">
.badgeGrid {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
  gap: var(--space-6);

  @media (max-width: 48rem) {
    grid-template-columns: 1fr;
  }
}

.controls {
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}

.colorRow {
  display: flex;
  align-items: center;
  gap: var(--space-2);

  input[type="color"] {
    width: 2.75rem;
    flex-shrink: 0;
  }
}

.checkboxLabel {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  color: var(--text);
}

.preview {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: var(--space-3);
}

.previewBadge {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  padding: var(--space-6);
  background-color: var(--bg-sunken);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  // The badge is a flat SVG; a checkerboard would only add noise.
  overflow-x: auto;
}
</style>
