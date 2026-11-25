// The HTML editor in each display mode (kit-editor's EditorFrame: Menus, Single line, Classic, and
// Focus), through the development host (dev.html): node tests/modes.mjs [chromium|webkit]
// kition-sdk's mode smoke (`@kition/sdk/dev-host/mode-smoke`). Run it with no HTTP proxy set. The
// Edit tab has three commands, all in Single line's row.
import { chromium, webkit } from '@playwright/test'
import { createServer } from 'vite'
import { runModeSmoke } from '@kition/sdk/dev-host/mode-smoke'

const line = { tab: 'Edit', primary: ['html-cmd-file.setSource'], min: 1 }
await runModeSmoke({ kit: 'kit.html', line, config: new URL('../vite.config.ts', import.meta.url).pathname, browser: process.argv[2] === 'webkit' ? webkit : chromium, createServer })
