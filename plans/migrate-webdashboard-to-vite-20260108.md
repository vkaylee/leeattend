# Plan: Migrate Webdashboard from Next.js to Vite + React

**Goal:** Convert the `webdashboard` application from Next.js (SSR) to a standard Vite + React SPA to simplify the architecture for Tauri integration.

**Date:** 2026-01-08
**Status:** Draft

## 1. Preparation & Safety
- [ ] **Backup:** Create a backup of `webdashboard/` or ensure git status is clean.
- [ ] **Analyze Routes:**
    - Root: `src/app/page.tsx` -> `/`
    - Agents: `src/app/agents/page.tsx` -> `/agents`
    - Devices: `src/app/devices/page.tsx` -> `/devices`
    - Layout: `src/app/layout.tsx` -> Root wrapper

## 2. Dependency Management
- [ ] **Remove Next.js:** Uninstall `next`, `eslint-config-next`.
- [ ] **Install Vite Stack:** Install `vite`, `@vitejs/plugin-react`, `react-router-dom`.
- [ ] **Update Scripts:** Update `package.json` scripts (`dev`, `build`, `start`) to use `vite`.

## 3. Configuration Setup
- [ ] **Vite Config:** Create `vite.config.ts` (configure aliases `@/*` to point to `src/*`).
- [ ] **TypeScript Config:** Update `tsconfig.json` (remove Next.js specific settings, ensure Vite compatibility).
- [ ] **Index HTML:** Create `index.html` in `webdashboard/` root as the entry point.

## 4. Code Migration & Refactoring
- [ ] **Entry Point:** Create `src/main.tsx` to render the React root.
- [ ] **Router Setup:** Create `src/App.tsx` (or `src/routes.tsx`) to define routes using `react-router-dom`.
    - Move logic from `src/app/layout.tsx` to `src/App.tsx` (wrapping the router/outlet).
- [ ] **Page Migration:**
    - Convert `src/app/page.tsx` -> `src/pages/Dashboard.tsx` (or similar).
    - Convert `src/app/agents/page.tsx` -> `src/pages/Agents.tsx`.
    - Convert `src/app/devices/page.tsx` -> `src/pages/Devices.tsx`.
- [ ] **Component Refactor:**
    - Replace `import Link from 'next/link'` with `import { Link } from 'react-router-dom'`.
    - Replace `import Image from 'next/image'` with standard `<img>` tags.
    - Remove `'use client'` directives.
    - Remove `export const metadata` (move title/meta to `index.html` or `useEffect`).

## 5. Tauri Integration Update
- [ ] **Tauri Config:** Update `src-tauri/tauri.conf.json`:
    - `frontendDist`: Change from `../out` to `../dist`.
    - `devUrl`: Ensure it matches Vite's default (usually `http://localhost:5173`).

## 6. Cleanup
- [ ] **Delete Next.js Artifacts:** Remove `next.config.ts`, `.next/`, `src/app/`.

## 7. Verification
- [ ] **Run Dev:** `npm run dev` works.
- [ ] **Navigation:** Clicking links works without full page reload.
- [ ] **Build:** `npm run build` generates `dist/`.
- [ ] **Tauri:** `npm run tauri dev` loads the app correctly.
