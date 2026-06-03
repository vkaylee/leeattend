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
      'src/routeTree.gen.ts',
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
      'unused-imports/no-unused-imports': 'error',
      'import-x/no-named-as-default-member': 'error',
      'unused-imports/no-unused-vars': [
        'error',
        {
          vars: 'all',
          varsIgnorePattern: '^_',
          args: 'after-used',
          argsIgnorePattern: '^_',
        },
      ],

      // --- Option A: Type-Aware Strictness ---
      '@typescript-eslint/no-floating-promises': 'error',
      '@typescript-eslint/await-thenable': 'error',
      '@typescript-eslint/no-misused-promises': [
        'error',
        { checksVoidReturn: { attributes: false } }
      ],
      '@typescript-eslint/no-unnecessary-condition': 'error',
      '@typescript-eslint/no-non-null-assertion': 'error',
      '@typescript-eslint/restrict-template-expressions': [
        'error',
        {
          allowNumber: true,
          allowBoolean: true,
          allowNullish: true,
        },
      ],

      // --- Option B: Pattern Consistency (Perfectionist & Unicorn) ---
      'perfectionist/sort-imports': [
        'error',
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
      'perfectionist/sort-jsx-props': ['error', { type: 'alphabetical', order: 'asc' }],
      'perfectionist/sort-interfaces': ['error', { type: 'alphabetical', order: 'asc' }],
      
      'unicorn/filename-case': ['error', { case: 'kebabCase' }],
      'unicorn/no-null': 'off',
      'unicorn/prevent-abbreviations': 'off', // Too pedantic and causes issues with standard names like 'e' or 'props'


      // --- Option C: Enterprise Hardening ---
      'complexity': ['error', 10],
      'max-lines-per-function': ['warn', { max: 150, skipBlankLines: true, skipComments: true }],
      'max-params': ['error', 4],
      'no-console': ['warn', { allow: ['warn', 'error'] }],
      
      '@typescript-eslint/naming-convention': 'off', // Often conflicts with API responses or generated code


      // Standard React Polish
      'react/jsx-no-leaked-render': ['error', { validStrategies: ['coerce', 'ternary'] }],
      'react/prop-types': 'off', // Using TS instead
      'react-refresh/only-export-components': ['warn', { allowConstantExport: true }],
      'react/forbid-elements': ['error', { forbid: [
        { element: 'h1', message: 'Use <Typography variant="h1"> instead.' },
        { element: 'h2', message: 'Use <Typography variant="h2"> instead.' },
        { element: 'h3', message: 'Use <Typography variant="h3"> instead.' },
        { element: 'h4', message: 'Use <Typography variant="h4"> instead.' },
        { element: 'h5', message: 'Use <Typography variant="h5"> instead.' },
        { element: 'h6', message: 'Use <Typography variant="h6"> instead.' },
        { element: 'p', message: 'Use <Typography variant="p"> instead.' }
      ] }],
    },
  },
  {
    files: ['src/**/*.{ts,tsx}'],
    ignores: [
      'src/components/ui/**/*.{ts,tsx}',
    ],
    rules: {
      'react/forbid-elements': ['error', { forbid: [
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
      'src/routeTree.gen.ts',
      'src/route-tree.gen.ts',
      'src/types/api-generated.ts',
      'src/lib/api-generated/**/*',
      // Legacy files violating max-lines limits (> 200 lines)
      'src/hooks/use-dashboard-layout.ts',
      'src/hooks/use-dashboard.ts',
      'src/hooks/use-ess.ts',
      'src/hooks/use-working-pattern-form.ts',
      'src/hooks/use-workspace.ts',
      'src/lib/api/attendance.ts',
      'src/lib/api/helpers.ts',
      'src/lib/api/leaves.ts',
      'src/lib/api/payroll.ts',
      'src/lib/api/types.ts',
      'src/lib/permission-groups.ts',
      'src/pages/attendance/hooks/use-working-patterns.ts',
    ],
    rules: {
      'max-lines': ['error', { max: 200, skipBlankLines: true, skipComments: true }],
    },
  },
  {
    files: ['src/**/*.tsx'],
    ignores: [
      'src/components/ui/**/*',
      'src/**/*.test.tsx',
      'src/**/__tests__/**/*',
      // Legacy files violating max-lines limits (> 300 lines)
      'src/components/approvals/bulk-upload-ot-dialog.tsx',
      'src/components/approvals/confirm-claims-dialog.tsx',
      'src/components/common/navigable-list.tsx',
      'src/components/debug/debug-toolbar.tsx',
      'src/components/features/attendance/attendance-import-dialog.tsx',
      'src/components/features/dashboard/turnover-analytics-chart.tsx',
      'src/components/features/employees/contract-form-modal.tsx',
      'src/components/features/employees/employee-detail-drawer.tsx',
      'src/components/features/reports/report-export-dialog.tsx',
      'src/components/layout/sidebar.tsx',
      'src/components/leave/leave-type-manager.tsx',
      'src/components/payroll/bonus-policy-manager.tsx',
      'src/components/payroll/payroll-rules-manager.tsx',
      'src/components/payroll/payroll-table.tsx',
      'src/components/shifts/monthly-roster-grid.tsx',
      'src/components/shifts/shift-timeline-preview.tsx',
      'src/components/shifts/weekly-roster-grid.tsx',
      'src/pages/approvals/components/behalf-assignments-tab.tsx',
      'src/pages/approvals/ot-approvals-page.tsx',
      'src/pages/monitor/jobs-table.tsx',
      'src/pages/onboarding-page.tsx',
      'src/pages/settings/time-settings.tsx',
      'src/pages/settings/workflow-editor-page.tsx',
      'src/pages/shifts/shift-editor-page.tsx',
      'src/pages/system-admin/tenants-page.tsx',
      'src/pages/working-patterns-editor-page.tsx',
    ],
    rules: {
      'max-lines': ['error', { max: 300, skipBlankLines: true, skipComments: true }],
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
    },
  }
);