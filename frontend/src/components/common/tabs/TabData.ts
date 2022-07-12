import { Ref } from "vue";

export interface TabData {
  currentTab: Ref<string>;
  update: (tab: string) => void;
}
