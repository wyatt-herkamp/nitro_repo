import { ref, watch } from "vue";

export type Theme = "light" | "dark";
export type ThemePreference = Theme | "system";

const STORAGE_KEY = "nitro-theme";

function systemTheme(): Theme {
  return window.matchMedia("(prefers-color-scheme: light)").matches ? "light" : "dark";
}

function stored(): ThemePreference {
  const value = localStorage.getItem(STORAGE_KEY);
  return value === "light" || value === "dark" || value === "system" ? value : "system";
}

const preference = ref<ThemePreference>(stored());
/** What is actually on screen — `preference` resolved through the OS setting. */
const active = ref<Theme>(preference.value === "system" ? systemTheme() : preference.value);

function apply() {
  active.value = preference.value === "system" ? systemTheme() : preference.value;
  // Absent when following the OS, so `tokens.scss` can fall back to the `prefers-color-scheme`
  // media query rather than needing this module to have run before first paint.
  if (preference.value === "system") {
    document.documentElement.removeAttribute("data-theme");
  } else {
    document.documentElement.setAttribute("data-theme", preference.value);
  }
}

// Following the OS means following it as it changes, not just at load.
window.matchMedia("(prefers-color-scheme: light)").addEventListener("change", () => {
  if (preference.value === "system") {
    apply();
  }
});

watch(preference, (value) => {
  localStorage.setItem(STORAGE_KEY, value);
  apply();
});

apply();

export function useTheme() {
  function set(value: ThemePreference) {
    preference.value = value;
  }

  /** Cycles dark → light → system, which is the whole interaction the nav button needs. */
  function cycle() {
    preference.value =
      preference.value === "dark" ? "light" : preference.value === "light" ? "system" : "dark";
  }

  return { preference, active, set, cycle };
}
