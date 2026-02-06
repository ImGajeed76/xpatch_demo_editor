# xpatch_demo_editor

[![GitHub](https://img.shields.io/github/license/imgajeed76/xpatch_demo_editor)](https://github.com/imgajeed76/xpatch_demo_editor/blob/main/LICENSE)
[![Built with Tauri](https://img.shields.io/badge/built%20with-Tauri-24C8DB?logo=tauri)](https://tauri.app)
[![AGPL v3](https://img.shields.io/badge/License-AGPL%20v3-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)

A lightning-fast markdown editor demo powered by [xpatch](https://github.com/ImGajeed76/xpatch) - watch it achieve
crazy compression while letting you fly through document history like you're scrubbing a video timeline. Built with
Tauri, SvelteKit, and magic.

## What is this?

This is a live demonstration of what happens when you take a delta compression library that can store entire code
repositories in a few bytes per change, and wrap it in a beautiful, snappy editor interface. Every keystroke becomes a
patch. Every patch is smaller than this sentence. And you can scrub through thousands of versions with zero perceptible
lag.

**Watch the demo:**
![Demo GIF](./assets/xpatch_demo.gif)
\* The scrubbing is smoother when you actually have the app open. I'm just incapable of holding my arrow keys down lol.

## Features

- **Blazing Fast Time Travel**: Scrub through document history with a slider or hold arrow keys to animate through
  versions at 20fps
- **Zero-Latency Navigation**: Arrow key navigation feels like scrubbing a video timeline
- **Smart Version Optimization**: Automatically finds the best base version to encode against (not just the previous
  one)
- **SQLite Backend**: Persistent storage with efficient patch retrieval
- **Beautiful UI**: shadcn-svelte components with markdown preview and syntax highlighting
- **Live Stats**: Real-time compression metrics showing bytes saved per patch

## The Magic

The secret sauce is **xpatch** analyzing every change and automatically picking the perfect compression algorithm. A
simple character insertion? That's 2 bytes. A complex refactor? It'll find the optimal base version from 16 commits back
and still compress it down to nearly nothing.

Then we cache reconstructed versions in memory, so when you're holding down the arrow key to animate through history,
you're not waiting for decompression—you're just pulling from a hot cache.

## Requirements

- **OS**: Windows 10+, macOS 10.15+, or modern Linux
- **Tauri**: https://v2.tauri.app/start/prerequisites/
- **Node**: 18+ (or bun for frontend build)

## Installation

```bash
# Clone the repository
git clone https://github.com/imgajeed76/xpatch_demo_editor.git
cd xpatch_demo_editor

# Install dependencies
bun install  # or npm install, pnpm install, etc.
```

## Quick Start

The easiest way to run it is with [axogen](https://github.com/axonotes/axogen) (my build tool that handles the
annoying Linux graphics sync issues for you):

```bash
# Using axogen (recommended)
axogen run dev
```

Or manually:

```bash
# Standard development
bun run tauri dev

# On Linux with explicit sync issues
__NV_DISABLE_EXPLICIT_SYNC=1 bun run tauri dev
```

That's it. The app will open, create a SQLite database in your app data directory, and you're ready to start typing.

## Usage

### Creating Documents

1. Give your new document a title
2. And click "Create"
3. It will open the editor

Your document is now saved automatically every 500ms as you type.

### Time Travel

Click the "Time Travel" button to open the version history panel:

- **Slider**: Jump to any version instantly
- **Arrow Keys**: Hold ← or → to animate through versions smoothly
- **Fine Controls**: Use the prev/next buttons for single-step navigation
- **Return to Current**: Jump back to the latest version

The UI shows you exactly which version you're viewing and when it was created.

### Markdown Preview

Toggle between "Write" and "Preview" modes to see rendered markdown with syntax-highlighted code blocks.

## License

This project is licensed under **AGPL-3.0-or-later**.

## Contributing

Contributions are welcome. Please open an issue or pull request on GitHub. This is a demo, but it's also a showcase of
what's possible with xpatch—if you have ideas for cooler features, let's build them.

## Related Projects

- [xpatch](https://github.com/ImGajeed76/xpatch) - The delta compression library powering this demo
- [axogen](https://github.com/axonotes/axogen) - Build tool that makes Tauri development smoother
- [gdelta](https://github.com/ImGajeed76/gdelta) - General-purpose delta compression used by xpatch
- [Check it out on my website](https://oseifert.ch/projects/xpatch-demo-editor-1115)
