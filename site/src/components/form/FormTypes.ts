export interface DropDownOption<T> {
  name: string;
  value: T;
}

export function enumToOptions(e: any): DropDownOption<string>[] {
  return Object.keys(e).map((k) => {
    return { name: k, value: e[k] };
  });
}
