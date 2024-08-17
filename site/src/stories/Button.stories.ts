import type { Meta, StoryObj } from '@storybook/vue3'
import { fn } from '@storybook/test'
import SubmitButton from '@/components/form/SubmitButton.vue'

// More on how to set up stories at: https://storybook.js.org/docs/writing-stories
const meta = {
  title: 'Forms/SubmitButton',
  component: SubmitButton,
  // This component will have an automatically generated docsPage entry: https://storybook.js.org/docs/writing-docs/autodocs
  tags: ['autodocs'],
  argTypes: {},
  args: {
    default: fn(() => 'Submit'),
    onClick: fn()
  }
} satisfies Meta<typeof SubmitButton>

export default meta
type Story = StoryObj<typeof meta>
/*
 *👇 Render functions are a framework specific feature to allow you control on how the component renders.
 * See https://storybook.js.org/docs/api/csf
 * to learn how to use render functions.
 */
export const Primary: Story = {
  args: {
    default: () => 'Submit'
  }
}
