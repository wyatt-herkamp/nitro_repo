<template>
  <section :class="['nCard', { isFlush: flush }]">
    <header
      v-if="title || $slots.header || $slots.actions"
      class="cardHeader">
      <div class="cardTitle">
        <h2 v-if="title">{{ title }}</h2>
        <slot name="header" />
        <p
          v-if="subtitle"
          class="cardSubtitle">
          {{ subtitle }}
        </p>
      </div>
      <div
        v-if="$slots.actions"
        class="cardActions">
        <slot name="actions" />
      </div>
    </header>

    <div :class="['cardBody', { isFlush: flush }]">
      <slot />
    </div>

    <footer
      v-if="$slots.footer"
      class="cardFooter">
      <slot name="footer" />
    </footer>
  </section>
</template>

<script setup lang="ts">
/** A bordered surface with an optional titled header, action slot and footer. */
withDefaults(
  defineProps<{
    title?: string;
    subtitle?: string;
    /** Drops the body padding, for cards that hold a table or a list flush to the edges. */
    flush?: boolean;
  }>(),
  { title: undefined, subtitle: undefined, flush: false },
);
</script>

<style scoped lang="scss">
.nCard {
  background-color: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  overflow: hidden;
}

.cardHeader {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--space-4);
  padding: var(--space-4) var(--space-5);
  border-bottom: 1px solid var(--border);
}

.cardTitle {
  min-width: 0;

  h2 {
    font-size: var(--text-md);
    font-weight: var(--weight-semibold);
  }
}

.cardSubtitle {
  margin-top: var(--space-1);
  font-size: var(--text-sm);
  color: var(--text-muted);
}

.cardActions {
  display: flex;
  flex-shrink: 0;
  gap: var(--space-2);
}

.cardBody {
  padding: var(--space-5);

  &.isFlush {
    padding: 0;
  }
}

.cardFooter {
  display: flex;
  justify-content: flex-end;
  gap: var(--space-2);
  padding: var(--space-3) var(--space-5);
  background-color: var(--bg-sunken);
  border-top: 1px solid var(--border);
}
</style>
