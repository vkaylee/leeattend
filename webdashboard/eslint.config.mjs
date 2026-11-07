import js from '@eslint/js';
import globals from 'globals';
import reactHooks from 'eslint-plugin-react-hooks';
import reactRefresh from 'eslint-plugin-react-refresh';
import tseslint from 'typescript-eslint';
import jsxA11y from 'eslint-plugin-jsx-a11y';
import importX from 'eslint-plugin-import-x';
import perfectionist from 'eslint-plugin-perfectionist';
import unicorn from 'eslint-plugin-unicorn';
import react from 'eslint-plugin-react';
import unusedImports from 'eslint-plugin-unused-imports';

export default tseslint.config(
  { 
    ignores: [
      'dist', 
      'playwright-report', 
      'test-results', 
      'coverage', 
      'node_modules', 
      'src/route-tree.gen.ts',
      'src/types/api-generated.ts',
      'src/lib/api-generated/**/*',
      'src/vite-env.d.ts',
      'src/components/shifts/__tests__/shift-modals.test.tsx',
      'scratch',
      'orval.config.ts',
      'playwright.config.ts',
      'playwright.dev.config.ts',
      'vitest.config.ts'
    ] 
  },
  {
    extends: [
      js.configs.recommended,
      ...tseslint.configs.strictTypeChecked,
      ...tseslint.configs.stylisticTypeChecked,
      importX.flatConfigs.recommended,
      importX.flatConfigs.typescript,
    ],
    files: ['**/*.{ts,tsx}'],
    languageOptions: {
      ecmaVersion: 2020,
      globals: globals.browser,
      parserOptions: {
        project: ['./tsconfig.json'],
        tsconfigRootDir: import.meta.dirname,
      },
    },
    settings: {
      'import-x/resolver': {
        typescript: true,
        node: true,
      },
      react: {
        version: 'detect',
      },
    },
    plugins: {
      'react-hooks': reactHooks,
      'react-refresh': reactRefresh,
      'jsx-a11y': jsxA11y,
      'perfectionist': perfectionist,
      'unicorn': unicorn,
      'react': react,
      'unused-imports': unusedImports,
    },
    rules: {
      ...reactHooks.configs.recommended.rules,
      ...jsxA11y.flatConfigs.recommended.rules,
      ...react.configs.recommended.rules,
      ...react.configs['jsx-runtime'].rules,

      '@typescript-eslint/no-unused-vars': 'off',
      'unused-imports/no-unused-imports': 'warn',
      'import-x/no-named-as-default-member': 'warn',
      'unused-imports/no-unused-vars': [
        'warn',
        {
          vars: 'all',
          varsIgnorePattern: '^_',
          args: 'after-used',
          argsIgnorePattern: '^_',
        },
      ],

      // --- Option A: Type-Aware Strictness ---
      '@typescript-eslint/no-floating-promises': 'warn',
      '@typescript-eslint/await-thenable': 'warn',
      '@typescript-eslint/no-misused-promises': [
        'warn',
        { checksVoidReturn: { attributes: false } }
      ],
      '@typescript-eslint/no-unnecessary-condition': 'warn',
      '@typescript-eslint/no-non-null-assertion': 'warn',
      '@typescript-eslint/restrict-template-expressions': [
        'warn',
        {
          allowNumber: true,
          allowBoolean: true,
          allowNullish: true,
        },
      ],

      // --- Option B: Pattern Consistency (Perfectionist & Unicorn) ---
      'perfectionist/sort-imports': [
        'warn',
        {
          type: 'alphabetical',
          order: 'asc',
          groups: [
            'type',
            'react',
            ['builtin', 'external'],
            'internal-type',
            'internal',
            ['parent-type', 'sibling-type', 'index-type'],
            ['parent', 'sibling', 'index'],
            'object',
            'unknown',
          ],
          customGroups: {
            value: {
              react: ['react', 'react-dom'],
            },
          },
          newlinesBetween: 'always',
        },
      ],
      'perfectionist/sort-jsx-props': ['warn', { type: 'alphabetical', order: 'asc' }],
      'perfectionist/sort-interfaces': ['warn', { type: 'alphabetical', order: 'asc' }],
      
      'unicorn/filename-case': ['warn', { case: 'kebabCase' }],
      'unicorn/no-null': 'off',
      'unicorn/prevent-abbreviations': 'off', // Too pedantic and causes issues with standard names like 'e' or 'props'


      // --- Option C: Enterprise Hardening ---
      'complexity': ['warn', 10],
      'max-lines-per-function': ['warn', { max: 150, skipBlankLines: true, skipComments: true }],
      'max-params': ['warn', 4],
      'no-console': ['warn', { allow: ['warn'] }],
      
      '@typescript-eslint/naming-convention': 'off', // Often conflicts with API responses or generated code


      // Standard React Polish
      'react/jsx-no-leaked-render': ['warn', { validStrategies: ['coerce', 'ternary'] }],
      'react/prop-types': 'off', // Using TS instead
      'react-refresh/only-export-components': ['warn', { allowConstantExport: true }],
      'react/forbid-elements': ['warn', { forbid: [
        { element: 'h1', message: 'Use <Typography variant="h1"> instead.' },
        { element: 'h2', message: 'Use <Typography variant="h2"> instead.' },
        { element: 'h3', message: 'Use <Typography variant="h3"> instead.' },
        { element: 'h4', message: 'Use <Typography variant="h4"> instead.' },
        { element: 'h5', message: 'Use <Typography variant="h5"> instead.' },
        { element: 'h6', message: 'Use <Typography variant="h6"> instead.' },
        { element: 'p', message: 'Use <Typography variant="p"> instead.' }
      ] }],

      // Additional warning levels to avoid test blocks on legacy code
      '@typescript-eslint/no-unsafe-assignment': 'error',
      '@typescript-eslint/no-unsafe-member-access': 'error',
      '@typescript-eslint/no-unsafe-call': 'error',
      '@typescript-eslint/no-unsafe-argument': 'error',
      '@typescript-eslint/no-unsafe-return': 'error',
      // PI 2: promoted from warn → error. Production src is now at 0 literal
      // `as any` (only the typed `client as unknown as` generic cast remains,
      // which this rule does not flag). Test files are still explicitly set to
      // 'off' by the test-files override lower in this config, so this only
      // hardens production code and blocks regressions from future `as any`.
      '@typescript-eslint/no-explicit-any': 'error',
      '@typescript-eslint/no-base-to-string': 'warn',
      '@typescript-eslint/no-confusing-void-expression': 'warn',
      'jsx-a11y/click-events-have-key-events': 'error',
      'jsx-a11y/no-static-element-interactions': 'error',
      'jsx-a11y/label-has-associated-control': 'error',
      '@typescript-eslint/require-await': 'warn',
      '@typescript-eslint/only-throw-error': 'warn',
      '@typescript-eslint/ban-ts-comment': 'warn',
      'react-hooks/error-boundaries': 'warn',
      'react-hooks/rules-of-hooks': 'warn',
      'react-hooks/exhaustive-deps': 'warn',
      '@typescript-eslint/no-dynamic-delete': 'warn',
      '@typescript-eslint/no-empty-function': 'warn',
      '@typescript-eslint/no-unnecessary-type-assertion': 'warn',
      '@typescript-eslint/no-unnecessary-type-conversion': 'warn',
      '@typescript-eslint/prefer-nullish-coalescing': 'warn',
      '@typescript-eslint/prefer-optional-chain': 'warn',
      '@typescript-eslint/use-unknown-in-catch-callback-variable': 'warn',
      '@typescript-eslint/no-deprecated': 'warn',
      'jsx-a11y/no-autofocus': 'error',
      'react/no-unescaped-entities': 'warn',
      'react-hooks/set-state-in-effect': 'warn',
      'react-hooks/preserve-manual-memoization': 'warn',
    },
  },
  {
    files: ['src/**/*.{ts,tsx}'],
    ignores: [
      'src/hooks/common/**/*.{ts,tsx}',
      'src/lib/query-keys.ts',
    ],
    rules: {
      'no-restricted-syntax': [
        'warn',
        {
          selector:
            'CallExpression[callee.type="Identifier"][callee.name=/^(useQuery|useInfiniteQuery)$/] > ObjectExpression > Property[key.name="queryKey"][value.type="ArrayExpression"]',
          message: 'Use a query-key factory instead of a literal queryKey.',
        },
        {
          selector:
            'CallExpression[callee.type="Identifier"][callee.name="useMutation"] > ObjectExpression > Property[key.name=/^(mutationKey|queryKey)$/][value.type="ArrayExpression"]',
          message: 'Use a query-key factory instead of a literal mutation key.',
        },
        {
          selector:
            'CallExpression[callee.type="MemberExpression"][callee.property.name="invalidateQueries"] > ObjectExpression > Property[key.name="queryKey"][value.type="ArrayExpression"]',
          message: 'Use a query-key factory instead of a literal queryKey.',
        },
      ],
    },
  },
  {
    files: ['src/**/*.{ts,tsx}'],
    ignores: [
      'src/components/ui/**/*.{ts,tsx}',
    ],
    rules: {
      'react/forbid-elements': ['warn', { forbid: [
        { element: 'h1', message: 'Use <Typography variant="h1"> instead.' },
        { element: 'h2', message: 'Use <Typography variant="h2"> instead.' },
        { element: 'h3', message: 'Use <Typography variant="h3"> instead.' },
        { element: 'h4', message: 'Use <Typography variant="h4"> instead.' },
        { element: 'h5', message: 'Use <Typography variant="h5"> instead.' },
        { element: 'h6', message: 'Use <Typography variant="h6"> instead.' },
        { element: 'p', message: 'Use <Typography variant="p"> instead.' },
        { element: 'button', message: 'Use <Button> instead.' }
      ] }],
    },
  },
  {
    files: ['src/**/*.ts'],
    ignores: [
      'src/components/ui/**/*',
      'src/**/*.test.ts',
      'src/**/__tests__/**/*',
      'src/route-tree.gen.ts',
      'src/types/api-generated.ts',
      'src/lib/api-generated/**/*',
      // Legacy files violating max-lines limits (> 200 lines) (.ts)

    ],
    rules: {
      'max-lines': ['warn', { max: 200, skipBlankLines: true, skipComments: true }],
    },
  },
  {
    files: ['src/**/*.tsx'],
    ignores: [
      'src/components/ui/**/*',
      'src/**/*.test.tsx',
      'src/**/__tests__/**/*',
      // Legacy files violating max-lines limits (> 300 lines)
      'src/components/features/attendance/attendance-import-dialog.tsx',
      'src/components/features/dashboard/turnover-analytics-chart.tsx',
      'src/components/features/reports/report-export-dialog.tsx',
      'src/components/leave/leave-type-manager.tsx',
      'src/components/shifts/weekly-roster-grid.tsx',
      'src/pages/monitor/jobs-table.tsx',
      'src/pages/shifts/shift-editor-page.tsx',
      // Pending refactor — >300 lines (tsx)
      'src/components/features/attendance/staging-logs-table.tsx',
      'src/components/features/employees/econtracts-table.tsx',
      'src/components/features/reports/daily-report-table.tsx',
      'src/components/members/member-table.tsx',
      'src/components/payroll/payroll-summary-table.tsx',
      'src/pages/approvals/components/ot-approvals-table.tsx',
      'src/pages/attendance/components/unrecorded-shifts/NoShowTable.tsx',
      'src/pages/departments-page.tsx',
      'src/pages/settings/components/econtracts-rules-tab.tsx',
      'src/pages/settings/custom-fields-page.tsx',
      'src/pages/shift-swaps-page.tsx',
      'src/pages/settings/condition-types-page.tsx',
      'src/pages/settings/custom-fields-page.tsx',
      'src/pages/shift-swaps-page.tsx',
      'src/pages/system-admin/smtp-settings-page.tsx',
    ],
    rules: {
      'max-lines': ['warn', { max: 300, skipBlankLines: true, skipComments: true }],
    },
  },
  {
    files: [
      '**/*.test.{ts,tsx}',
      '**/__tests__/**/*.{ts,tsx}',
      'tests/**/*.ts',
      'tests/**/*.tsx',
      'orval.config.ts',
      'playwright.config.ts',
      'playwright.dev.config.ts',
      'vite.config.ts',
      'scratch/**/*.ts',
      'vitest.config.ts'
    ],
    extends: [tseslint.configs.disableTypeChecked],
    languageOptions: {
      parserOptions: {
        project: null,
      },
    },
  },
  {
    files: [
      '**/*.test.{ts,tsx}',
      '**/__tests__/**/*.{ts,tsx}',
      'tests/**/*.ts',
      'tests/**/*.tsx'
    ],
    rules: {
      '@typescript-eslint/no-explicit-any': 'off',
      '@typescript-eslint/no-non-null-assertion': 'off',
      '@typescript-eslint/no-unnecessary-condition': 'off',
      '@typescript-eslint/no-unsafe-assignment': 'off',
      '@typescript-eslint/no-unsafe-member-access': 'off',
      '@typescript-eslint/no-unsafe-call': 'off',
      '@typescript-eslint/no-unsafe-return': 'off',
      '@typescript-eslint/no-unsafe-argument': 'off',
      'max-lines-per-function': 'off',
      'complexity': 'off',
      // jsx-a11y rules are for runtime DOM; test files use testing-library
      // which has its own a11y assertions via jest-dom.
      'no-console': 'off',
      'jsx-a11y/click-events-have-key-events': 'off',
      'jsx-a11y/no-static-element-interactions': 'off',
      'jsx-a11y/label-has-associated-control': 'off',
      'jsx-a11y/no-autofocus': 'off',
    },
  }
);