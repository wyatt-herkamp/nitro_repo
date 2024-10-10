import pluginVue from 'eslint-plugin-vue'
import eslintConfigPrettier from 'eslint-config-prettier'
import vueTsEslintConfig from '@vue/eslint-config-typescript'
import compat from 'eslint-plugin-compat'
import skipFormattingConfig from '@vue/eslint-config-prettier/skip-formatting'

export default [
    ...pluginVue.configs['flat/essential'],
    eslintConfigPrettier,
    ...vueTsEslintConfig(),
    skipFormattingConfig,
    compat.configs['flat/recommended'],
    {
        rules: {
            'vue/multi-word-component-names': 'off',
            '@typescript-eslint/no-explicit-any': 'off',
        },
    },
]
