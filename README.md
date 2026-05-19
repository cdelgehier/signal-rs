# Signal RS

[![Tauri](https://img.shields.io/badge/Tauri_2-FFC131.svg?style=for-the-badge&logo=Tauri&logoColor=black)](https://tauri.app/)
[![Vue](https://img.shields.io/badge/Vue_3-4FC08D.svg?style=for-the-badge&logo=Vue.js&logoColor=white)](https://vuejs.org/)
[![Nuxt](https://img.shields.io/badge/Nuxt_4-00DC82.svg?style=for-the-badge&logo=Nuxt.js&logoColor=white)](https://nuxt.com/)
[![Nuxt UI](https://img.shields.io/badge/Nuxt_UI_4-00DC82.svg?style=for-the-badge&logo=Nuxt.js&logoColor=white)](https://ui.nuxt.com/)
[![GitHub Actions](https://img.shields.io/badge/GitHub_Actions-2088FF.svg?style=for-the-badge&logo=GitHub-Actions&logoColor=white)](https://github.com/features/actions)
[![pre-commit](https://img.shields.io/badge/pre--commit-enabled-FAB040.svg?style=for-the-badge&logo=pre-commit&logoColor=black)](https://pre-commit.com/)

A Signal desktop chat client built with Tauri 2 and [presage](https://github.com/whisperfish/presage), inspired by [gurk-rs](https://github.com/boxdot/gurk-rs). Replaces the terminal UI with a modern Nuxt 4 + Vue 3 interface.

## Features

- **QR code pairing** — link as a secondary device from your phone
- **Automatic re-pairing** — detects deassociation and shows a new QR code
- **Real-time messaging** — send and receive messages via Signal's WebSocket
- **Contact list** — contacts and groups synced from the local store, with last message preview (full sync requires receiving `Contacts` from the primary device)
- **Message history** — full conversation history from the local encrypted store
- **Multiline input** — Shift+Enter for new line, Enter to send
- **Dark theme** — Signal-inspired dark UI with Nuxt UI 4

## Architecture

```mermaid
flowchart TB
    subgraph FE["Frontend — Nuxt 4 + Vue 3"]
        direction TB
        Pages["Pages onboarding.vue · index.vue"]
        Components["Components QrCodeView · AppSidebar MessageList · MessageBubble · MessageInput"]
        DDD["DDD Layer domains → application → infrastructure"]
    end

    subgraph IPC["Tauri IPC"]
        direction LR
        Invoke["invoke(command) → get_channels, send_message    generate_qr_code, get_messages"]
        Events["emit(event) ← contacts-updated   message-received · deassociated"]
    end

    subgraph Rust["Rust Backend — Tauri 2"]
        direction TB
        Handle["ManagerHandle Send + Sync mpsc::Sender + watch::Receiver"]
        subgraph Actor["Actor Thread — 32 MB stack, LocalSet"]
            Loop["actor_loop plain loop, no recursion"]
            Select["receive_loop tokio::select! stream ↔ request channel"]
        end
    end

    subgraph Signal["Signal Protocol"]
        direction TB
        Presage["presage Manager‹SqliteStore, Registered›"]
        DB[("SQLite + SQLcipher signal.db")]
        WS["receive_messages() WebSocket stream"]
        Server(["Signal Servers"])
    end

    Pages & Components --> DDD
    DDD -->|"invoke()"| Invoke --> Handle --> Loop
    Loop --> Select
    Select -->|"dispatch_linked"| Presage
    Presage <--> DB
    Presage -->|"send_message"| Server
    Server -->|"incoming messages"| WS --> Select
    Select -->|"Tauri events"| Events --> DDD
```

## Getting started

```bash
# Install dependencies
task install

# Generate icons (requires src-tauri/icons/app-icon.png 1024×1024)
task icons

# Start in dev mode (Nuxt hot-reload + Tauri)
task up

# Run tests
task test

# Lint
task lint
```

On first launch, scan the QR code with Signal on your phone:
**Signal → Settings → Linked devices → Link a device**

## Available tasks

| Task | Description |
|------|-------------|
| `task up` | Start dev mode (Nuxt + Tauri with hot-reload) |
| `task test` | Run all tests (Rust unit/integration + Vitest) |
| `task lint` | Check Rust (clippy + rustfmt) + TypeScript |
| `task package` | Build production bundle (.dmg / .exe) |
| `task clean` | Remove all build artifacts |
| `task icons` | Regenerate icons from `src-tauri/icons/app-icon.png` |

## Contributing

### Setup

```bash
# Install pre-commit hooks
pre-commit install
pre-commit install --hook-type commit-msg
```

### Commit convention

This project enforces **[Conventional Commits](https://www.conventionalcommits.org/)** via [Commitizen](https://commitizen-tools.github.io/commitizen/).

```
<type>(<scope>): <short description>

Types: feat, fix, docs, refactor, test, chore, ci
```

Examples:

```bash
feat(signal): add group message support
fix(ui): scroll to bottom on new message
docs: update architecture diagram
```

Use `cz commit` for an interactive prompt, or write the message manually — the pre-commit hook will validate it.
