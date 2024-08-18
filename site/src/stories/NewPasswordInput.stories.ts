import type { Meta, StoryObj } from '@storybook/vue3'
import { fn } from '@storybook/test'
import NewPasswordInput from '@/components/form/text/NewPasswordInput.vue'

// More on how to set up stories at: https://storybook.js.org/docs/writing-stories
const meta = {
  title: 'Forms/NewPasswordInput',
  component: NewPasswordInput,
  // This component will have an automatically generated docsPage entry: https://storybook.js.org/docs/writing-docs/autodocs
  tags: ['autodocs'],
  argTypes: {
    modelValue: { control: { type: 'text' } }
  },
  args: {
    id: 'password',
    passwordRules: {
      min_length: 8,
      require_uppercase: true,
      require_lowercase: true,
      require_number: true,
      require_special: true
    }
  }
} satisfies Meta<typeof NewPasswordInput>

export default meta
type Story = StoryObj<typeof meta>
/*
 *👇 Render functions are a framework specific feature to allow you control on how the component renders.
 * See https://storybook.js.org/docs/api/csf
 * to learn how to use render functions.
 */
export const Primary: Story = {}
