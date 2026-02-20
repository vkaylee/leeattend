# LeeAttend Web Dashboard

A centralized management dashboard for the LeeAttend system, built with React + Vite + Tailwind CSS and integrated with Tauri for desktop application packaging.

## Technology Stack

- **React 19**: The core UI library.
- **Vite**: Extremely fast build tool and dev server.
- **React Router 7**: SPA navigation management.
- **Tailwind CSS 4**: Utility-first CSS framework.
- **Lucide React**: Beautiful icon set.
- **TanStack Query**: Server state and caching management.
- **Tauri**: Rust-based desktop application packaging.

## Development Guide

### 1. Installation

```bash
npm install
```

### 2. Configuration

Copy the `.env.example` file to `.env.local` and adjust `VITE_API_URL` to point to the address of the API Server (default is `http://localhost:3000`).

### 3. Run Development Environment

```bash
# Run web dashboard in the browser (default port 1420)
npm run dev

# Run Desktop application (Tauri)
npm run tauri dev
```

### 4. Build Application

```bash
# Build static web
npm run build

# Build Desktop application (Tauri)
npm run tauri build
```

## Directory Structure

- `src/app.tsx`: Main Router and Layout configuration.
- `src/pages/`: Pages (Dashboard, Agents, Devices, etc.).
- `src/components/`: Shared components.
- `src/lib/api.ts`: API call functions to apiserver.
- `src-tauri/`: Rust source code for the Desktop application.
