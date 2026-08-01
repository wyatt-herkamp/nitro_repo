export interface TabData {
  changeTab(tab: string): void;
  getTab(): string;
  isTabActive(tab: string): boolean;
}
