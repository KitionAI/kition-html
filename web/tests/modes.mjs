// The HTML editor in each display mode (kit-editor's EditorFrame: Menus, Single line, Classic, and
// Focus), through the development host (dev.html): node tests/modes.mjs [chromium|webkit]
// kition-sdk's mode smoke (`@kition/kit-dev-host`'s modeSmoke.mjs) from the checkout beside this
// repository (or KITION_SDK). Run it with no HTTP proxy set. The Edit tab has three commands, all
// in Single line's row.
import { chromium, webkit } from '@playwright/test'
import { createServer } from 'vite'

const sdk = process.env.KITION_SDK ?? new URL('../../../kition-sdk', import.meta.url).pathname
const { runModeSmoke } = await import(new URL('packages/kit-dev-host/src/modeSmoke.mjs', `file://${sdk}/`).href)
const line = { tab: 'Edit', primary: ['html-cmd-html.setSource'], min: 1 }
await runModeSmoke({ kit: 'kit.html', line, config: new URL('../vite.config.ts', import.meta.url).pathname, browser: process.argv[2] === 'webkit' ? webkit : chromium, createServer })
