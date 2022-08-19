export interface MenuItemType {
  index: string;
  active: boolean;
}

export interface MenuProvider {
  items: Record<string, MenuItemType>;
  activeIndex?: string;
  addItem: (item: MenuItemType) => void;
  removeItem: (item: MenuItemType) => void;
  onClick: (item: string) => void;
}
