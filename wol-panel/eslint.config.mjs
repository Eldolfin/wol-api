// @ts-check
import { Linter } from 'eslint'
import withNuxt from './.nuxt/eslint.config.mjs'

export default withNuxt(
  {
    rules: {
      '@typescript-eslint/switch-exhaustiveness-check': 'warn',
    },
    files: ['*.tx', '*.vue'],
    ignores: ['*.d.ts'],
    languageOptions: {
      parserOptions: {
        project: ['./tsconfig.json'],
      },
    },
  },
  // Your custom configs here
)
