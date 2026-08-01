<template>
  <nav class="sideNav">
    <slot />
  </nav>
</template>

<style scoped lang="scss">
.sideNav {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
  flex-shrink: 0;
  width: var(--sidebar-width);
  padding: var(--space-4) var(--space-3);
  background-color: var(--surface);
  border-right: 1px solid var(--border);
  // Follows the page rather than floating in a fixed-height box, and stays put while the content
  // beside it scrolls.
  position: sticky;
  top: var(--nav-height);
  align-self: flex-start;
  max-height: calc(100vh - var(--nav-height));
  overflow-y: auto;
}

// Below tablet the sidebar becomes a horizontal strip above the content rather than eating half the
// width of the screen.
@media (max-width: 48rem) {
  .sideNav {
    position: static;
    flex-direction: row;
    flex-wrap: wrap;
    width: 100%;
    max-height: none;
    border-right: none;
    border-bottom: 1px solid var(--border);
  }
}
</style>
