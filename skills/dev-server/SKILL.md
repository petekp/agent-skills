---
name: dev-server
description: Start development servers with intelligent port management. Use when asked to "start the dev server", "run dev", "start development", "launch the server", or any request to run a local development server. Handles port conflicts, detects project type, cleans up stale processes, and opens the browser automatically.
license: MIT
metadata:
  author: petekp
  version: "0.1.0"
---

# Dev Server

Start development servers with port conflict resolution, process cleanup, and automatic browser opening.

## Workflow

1. **Scan ports** using `scripts/check_ports.sh --scan`
2. **If ports are in use**, show the user which processes are running and ask if they want to kill them
3. **Kill stale processes** if requested using `scripts/check_ports.sh --kill <port>`
4. **Detect project type** by checking for package.json, then determine package manager
5. **Start the dev server** with the `--open` flag to auto-open browser

## Port Checking Script

```bash
# Scan common dev ports (3000, 3001, 5173, 8080, etc.)
./scripts/check_ports.sh --scan

# Check specific port
./scripts/check_ports.sh 3000

# Find first available port starting from 3000
./scripts/check_ports.sh --find 3000

# Kill processes on a port
./scripts/check_ports.sh --kill 3000

# List running node/dev processes
./scripts/check_ports.sh --list
```

## Detecting Project Type and Package Manager

Check in order:
1. `bun.lockb` → use `bun run dev`
2. `pnpm-lock.yaml` → use `pnpm dev`
3. `yarn.lock` → use `yarn dev`
4. `package-lock.json` or `package.json` → use `npm run dev`

For Next.js/Vite/etc., the `dev` script in package.json handles the specifics.

## Starting with Custom Port

If the default port is unavailable, use the `--port` flag:

```bash
npm run dev -- --port 3001
pnpm dev --port 3001
yarn dev --port 3001
bun run dev --port 3001
```

## Opening Browser Automatically

Always include the `--open` flag when starting the dev server. Most frameworks support this natively:

```bash
npm run dev -- --open
pnpm dev --open
yarn dev --open
bun run dev --open
```

For frameworks without `--open` support, open the browser manually after the server starts:

```bash
open http://localhost:3000  # macOS
```

**Combined example** (custom port + auto-open):
```bash
npm run dev -- --port 3001 --open
```

## Quick Reference

| Situation | Action |
|-----------|--------|
| Port 3000 in use | Kill process or use `--port 3001` |
| Multiple node processes | Run `--list` to identify, kill stale ones |
| Unknown project type | Check package.json scripts for dev command |
| Open browser | Use `--open` flag or `open http://localhost:PORT` |
