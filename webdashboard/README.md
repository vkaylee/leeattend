# LeeAttend Web Dashboard

Centralized management dashboard for LeeAttend, built with React, Vite, and Tailwind CSS.

## Technology Stack

- React 19
- Vite
- React Router 7
- Tailwind CSS 4
- Lucide React
- TanStack Query

## Development Guide

### Installation

```bash
bun install
```

### Configuration

Copy `.env.example` to `.env.local` and set `VITE_API_URL` only when deployment does not provide a reverse proxy.

### Run Development Environment

```bash
bun run dev
```

### Build Application

```bash
bun run build
```

## Directory Structure

- `src/app.tsx`: Main router and layout configuration.
- `src/pages/`: Application pages.
- `src/components/`: Shared components.
- `src/lib/api.ts`: API client functions for `apiserver`.
