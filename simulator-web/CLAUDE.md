# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is the frontend web application for the TCP Protocol Simulator system. It provides a visual interface to manage TCP simulators that emulate various industrial device protocols.

**Tech Stack:** Vue 3, TypeScript, Vite, Element Plus, Pinia, Axios

## Development Commands

```bash
# Install dependencies
npm install

# Start development server (port 3000)
npm run dev

# Build for production
npm run build

# Preview production build
npm run preview

# Lint and fix
npm run lint
```

## Backend Requirement

The app requires the dm-rust backend running on port 8080. Vite proxies `/lspcapi` requests to `http://localhost:8080`.

## Architecture

### Data Flow

```
Views → Stores (Pinia) → API Layer → Backend
```

- **Views** (`src/views/`): Page components that display UI and handle user interactions
- **Stores** (`src/stores/simulator.ts`): Centralized state management using Pinia composition API
- **API** (`src/api/`): Axios wrapper with response interceptors for error handling

### API Response Format

Backend returns unified JSON format:
```json
{
  "state": 0,       // 0 = success, non-zero = error
  "message": "...",
  "data": { ... }
}
```

The axios interceptor in `src/api/index.ts` handles error display via Element Plus ElMessage.

### Key Types

Types mirror the Rust backend structs in `src/types/simulator.ts`:
- `SimulatorInfo`: Full simulator details including state and stats
- `ProtocolInfo`: Available protocol metadata
- `CreateSimulatorRequest`: Parameters for creating new simulators

### Store Pattern

The simulator store uses Pinia's composition API:
- Reactive state: `simulators`, `protocols`, `currentSimulator`, `loading`
- Computed: `runningCount`, `stoppedCount`, `totalConnections`
- Actions wrap API calls and update local state

### Path Alias

`@/` maps to `src/` directory (configured in `vite.config.ts` and `tsconfig.json`).

## API Endpoints

All endpoints prefixed with `/lspcapi/tcp-simulator`:

| Method | Path | Description |
|--------|------|-------------|
| GET | `/protocols` | List supported protocols |
| POST | `/create` | Create simulator |
| GET | `/list` | List all simulators |
| GET | `/:id` | Get simulator details |
| DELETE | `/:id` | Delete simulator |
| POST | `/:id/start` | Start simulator |
| POST | `/:id/stop` | Stop simulator |
| POST | `/:id/state` | Update simulator state |
| POST | `/:id/fault` | Trigger fault simulation |
| POST | `/:id/clear-fault` | Clear fault |
| POST | `/:id/online` | Set online/offline status |
