<template>
  <Milkdown />
</template>

<script lang="ts" setup>
import { Editor, rootCtx, defaultValueCtx } from '@milkdown/core'
import { nord } from '@milkdown/theme-nord'
import { Milkdown, useEditor } from '@milkdown/vue'
import { commonmark } from '@milkdown/preset-commonmark'
import { listener, listenerCtx } from '@milkdown/plugin-listener'
const props = defineProps({
  value: {
    type: String,
    required: true
  }
})
const emit = defineEmits<{
  (e: 'update', value: string): void
}>()
function YourMarkdownUpdater(markdown: string) {
  emit('update', markdown)
}
const { get } = useEditor((root) =>
  Editor.make()
    .config((ctx) => {
      ctx.set(rootCtx, root)
      ctx.set(defaultValueCtx, props.value)
      const listener = ctx.get(listenerCtx)

      listener.markdownUpdated((ctx, markdown, prevMarkdown) => {
        if (markdown !== prevMarkdown) {
          YourMarkdownUpdater(markdown)
        }
      })
    })
    .use(listener)
    .config(nord)
    .use(commonmark)
)
</script>
<style lang="scss">
#markdownEditor {
  height: 100%;
  width: 100%;
}
</style>
