import { RustIcon } from "vue3-simple-icons";
import CargoProjectHelper from "./CargoProjectHelper.vue";
import type { FrontendRepositoryType } from "@/types/repository";

export const CargoFrontendDefinition = {
  name: "cargo",
  properName: "Cargo",
  projectComponent: {
    component: CargoProjectHelper,
    props: {},
  },
  icons: [
    {
      name: "Rust",
      component: RustIcon,
      url: "https://doc.rust-lang.org/cargo/",
      props: {},
    },
  ],
} as FrontendRepositoryType;

/**
 * Mirrors `CargoRegistryConfig` on the backend. Only `Hosted` exists; a `Proxy` variant would be
 * added here at the same time as the server's.
 */
export type CargoConfigType = {
  type: "Hosted";
};
