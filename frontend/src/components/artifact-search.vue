<template>
  <form>
    <label for="default-search">Artifact Search(AQL)</label>
    <div class="relative">
      <input
        type="search"
        id="default-search"
        placeholder="storage: public; repository: maven; groupID: me.kingtux"
        required
      />
      <button type="submit" class="">Search</button>
    </div>
  </form>
</template>
<style lang="css" scoped>
label {
  @apply mb-2;
  @apply text-sm;
  @apply font-medium;
  @apply text-gray-900;
  @apply sr-only;
  @apply dark:text-gray-300;
}
input {
  @apply block;
  @apply p-4;
  @apply pl-10;
  @apply w-full;
  @apply text-sm text-gray-900;
  @apply bg-gray-50;
  @apply rounded-lg;
  @apply border border-gray-300;
  @apply focus:ring-blue-500 focus:border-blue-500;
  @apply dark:bg-gray-700;
  @apply dark:border-gray-600;
  @apply dark:placeholder-gray-400;
  @apply dark:text-white;
  @apply dark:focus:ring-blue-500;
  @apply dark:focus:border-blue-500;
}
button {
  @apply text-white;
  @apply absolute;
  @apply right-2.5;
  @apply bottom-2.5;
  @apply bg-blue-700;
  @apply hover:bg-blue-800;
  @apply focus:ring-4 focus:outline-none focus:ring-blue-300;
  @apply font-medium;
  @apply rounded-lg;
  @apply text-sm;
  @apply px-4;
  @apply py-2;
  @apply dark:bg-blue-600;
  @apply dark:hover:bg-blue-700;
  @apply dark:focus:ring-blue-800;
}
</style>
