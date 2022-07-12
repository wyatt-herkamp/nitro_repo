<template>
  <form>
    <label for="search">Artifact Search(AQL)</label>
    <div class="relative">
      <input
        type="search"
        id="search"
        placeholder="storage: public; repository: maven; groupID: me.kingtux"
        required
      />
      <button type="submit">Search</button>
    </div>
  </form>
</template>
<style lang="css" scoped>
label {
  @apply mb-2;
  @apply text-sm;
  @apply font-medium;
  @apply text-secondary;
  @apply sr-only;
}
input {
  @apply block;
  @apply p-4;
  @apply pl-10;
  @apply w-full;
  @apply text-sm;
  @apply text-quaternary;
  @apply rounded-lg;
  @apply bg-secondary/25;
  @apply border-2;
  @apply border-accent/10;
}
button {
  @apply text-quaternary;
  @apply absolute;
  @apply right-2.5;
  @apply bottom-2.5;
  @apply bg-secondary;
  @apply hover:bg-secondary/50;
  @apply font-medium;
  @apply rounded-lg;
  @apply text-sm;
  @apply px-4;
  @apply py-2;
}
</style>
