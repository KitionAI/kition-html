// web/scripts/readme-shots.mjs: the README pictures of Kition HTML (assets/readme/<name>.png at the
// repository root). Every shot opens the showcase sample "Bakery Newsletter Email" in the
// development host (web/dev.html), in Classic display mode, light theme, English, at 1440 × 900
// and device scale 2, and changes it only through the editor.
//
//   node web/scripts/readme-shots.mjs [name ...]   # every shot, or only the named ones
//
// It follows the rules of kition-sdk's shared runner (tools/readme-shots) without using it: that
// runner takes Playwright from web/node_modules, and this repository installs @playwright/test at
// its root instead, as web/tests/modes.mjs uses it. Vite comes from web/node_modules.
//
// The sample comes from a kition-showcase checkout next to this repository (or the one
// KITION_SHOWCASE names). It needs the engine built in web/public/engine (pnpm engine) and no HTTP
// proxy set. When pngquant is installed each picture is shrunk with --quality 70-90; a README
// picture is at most 400 KB.
import { execFileSync } from 'node:child_process'
import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { chromium } from '@playwright/test'
import { createServer } from 'vite'

const web = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const repo = path.resolve(web, '..')
const out = path.join(repo, 'assets', 'readme')
const showcase = path.resolve(process.env.KITION_SHOWCASE ?? path.join(repo, '..', 'kition-showcase'))
const sample = path.join(showcase, 'html', 'bakery-newsletter-email', 'bakery-newsletter-email.html')
const VIEWPORT = { width: 1440, height: 900 }
const MAX_BYTES = 400 * 1024

/** Waits until the development host's status line (#status) includes `text`. */
const status = (page, text) => page.waitForFunction((part) => document.getElementById('status')?.textContent?.includes(part) ?? false, text, { timeout: 90_000 })

/** The textarea of the source, and its text. */
const source = (kit) => kit.getByTestId('html-source')

/** Selects `part` of the source in the textarea, as a person's drag would. */
async function selectInSource(kit, part) {
  await source(kit).evaluate((area, text) => {
    const at = area.value.indexOf(text)
    if (at < 0) throw new Error(`no ${JSON.stringify(text)} in the source`)
    area.focus()
    area.setSelectionRange(at, at + text.length)
    // Scroll the selection into view: the line's offset over the line height.
    const line = area.value.slice(0, at).split('\n').length - 1
    area.scrollTop = Math.max(0, line * parseFloat(getComputedStyle(area).lineHeight || '18') - area.clientHeight / 3)
  }, part)
}

/** Lets the engine take the typed source, run the check and draw the preview again. */
const settled = (page) => page.waitForTimeout(2000)

const SHOTS = [
  {
    // The whole editor: the newsletter's source beside its rendered preview.
    name: 'hero',
  },
  {
    // Editing: a new headline typed over the old one in the source; the preview has redrawn it.
    name: 'see-1',
    steps: async (page, kit) => {
      await selectInSource(kit, 'Rye season is back, and so is the Saturday queue')
      await page.keyboard.type('Rye season is back: book your Saturday loaf')
      await settled(page)
    },
  },
  {
    // An agent's edit, live: the host's agent (ai/run, as Kition's agent edits an open file)
    // replaces the source with the header's color changed; the source and the preview follow.
    name: 'see-2',
    steps: async (page, kit) => {
      const before = await source(kit).inputValue()
      const after = before.replaceAll('#8c4a2f', '#2f5d50').replaceAll('#f5c9a5', '#bfe3d0')
      await page.evaluate(async (text) => window.devHost.connection().ai.run([{ command: 'html.setSource', params: { source: text } }], false), after)
      await settled(page)
      await selectInSource(kit, '#2f5d50')
    },
  },
  {
    // The check: a picture the kit cannot load typed into the source, and the diagnostic for it.
    name: 'see-3',
    steps: async (page, kit) => {
      await selectInSource(kit, 'Hello neighbour,')
      await page.keyboard.press('ArrowLeft')
      await page.keyboard.type('<img src="images/rye-loaf.jpg" alt="A rye loaf" width="536"><br>')
      await settled(page)
    },
  },
  {
    // Focus: only the source and the preview, chosen from the display mode menu.
    name: 'see-4',
    steps: async (page, kit) => {
      await kit.locator('.kui-mode-button').click()
      await kit.locator('[data-testid$="-mode-focus"]').click()
      await page.mouse.move(5, VIEWPORT.height - 5)
      await page.waitForTimeout(1000)
    },
  },
]

const names = process.argv.slice(2)
const unknown = names.filter((name) => !SHOTS.some((shot) => shot.name === name))
if (unknown.length > 0) throw new Error(`no shot named ${unknown.join(', ')}; the shots are ${SHOTS.map((shot) => shot.name).join(', ')}`)
const wanted = names.length === 0 ? SHOTS : SHOTS.filter((shot) => names.includes(shot.name))
if (!fs.existsSync(sample)) throw new Error(`the sample is missing: ${sample}`)
fs.mkdirSync(out, { recursive: true })

const server = await createServer({ root: web, configFile: path.join(web, 'vite.config.ts'), server: { port: 0, host: '127.0.0.1' }, logLevel: 'error' })
await server.listen()
const origin = `http://127.0.0.1:${server.httpServer.address().port}`
const browser = await chromium.launch(process.env.CHROME ? { executablePath: process.env.CHROME } : {})
const written = []
const failed = []
try {
  for (const shot of wanted) {
    const context = await browser.newContext({ viewport: VIEWPORT, deviceScaleFactor: 2 })
    try {
      const page = await context.newPage()
      page.on('pageerror', (error) => console.log(`${shot.name}: page error: ${error.message}`))
      await page.goto(`${origin}/dev.html?locale=en-US&modes=classic,line,menus,focus`)
      // The host opens its own sample first; the showcase sample replaces it.
      await status(page, ' open in ')
      const name = path.basename(sample)
      await page.evaluate(async ({ name, text }) => window.devHost.open(name, new TextEncoder().encode(text).buffer), { name, text: fs.readFileSync(sample, 'utf8') })
      await status(page, `${name} open in `)
      const kit = page.frameLocator('#kit')
      await kit.locator('[data-testid="html-preview"] canvas').first().waitFor({ timeout: 90_000 })
      await page.waitForTimeout(2500)
      await shot.steps?.(page, kit)
      await page.waitForTimeout(800)
      const file = path.join(out, `${shot.name}.png`)
      await page.locator('#kit').screenshot({ path: file })
      written.push(file)
      console.log(`took ${shot.name}`)
    } catch (error) {
      failed.push(`${shot.name}: ${String(error?.message ?? error).split('\n')[0]}`)
      console.log(`failed ${failed.at(-1)}`)
    } finally {
      await context.close()
    }
  }
} finally {
  await browser.close()
  await server.close()
}

for (const file of written) {
  try {
    execFileSync('pngquant', ['--quality', '70-90', '--force', '--skip-if-larger', '--ext', '.png', file], { stdio: 'ignore' })
  } catch (error) {
    if (error.code === 'ENOENT') break
  }
}
for (const file of written) {
  const bytes = fs.statSync(file).size
  console.log(`${path.relative(repo, file).split(path.sep).join('/')}  ${Math.ceil(bytes / 1024)} KB${bytes > MAX_BYTES ? '  OVER 400 KB' : ''}`)
}
if (failed.length > 0) throw new Error(`${failed.length} of ${wanted.length} shots failed:\n  ${failed.join('\n  ')}`)
