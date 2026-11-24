// The page on the host's stored replica (kition-sdk Kit UI protocol 7 and UI-R1; an HTML file holds
// the page, the host keeps its replica beside it): node tests/history.mjs [chromium|webkit].
// Opens the sample under the development host with the replica store on and the engine's merge
// declared (`?replica=1&replica-merges-file=1`), edits the source, saves, and finds the replica
// stored; reloads the document as its save left it (`devHost.reload()`), which opens on the stored
// replica: the edit is there and the replica's id is the same. Then another program changes the
// file (`devHost.reopenChanged(bytes)`): the editor opens it with the stored replica, shows the
// outside change, keeps the replica's id, and the change is one `system` `import` change on the
// replica's history (read from the replica stored at the next save). Last, the file opened alone
// (`devHost.open`) is a replica of its own.
// Undo is not checked across a reload: undo steps live in the session (kition-kdm's UndoManager),
// not in the replica, so a session opened on the stored replica starts with none.
// Needs the engine and fonts built into public/ (pnpm run engine). Run it with no HTTP proxy set.
import { chromium, webkit } from '@playwright/test'
import { createServer } from 'vite'
import { Replica } from '@kition/sdk/kdm'

const which = process.argv[2] ?? 'chromium'
const FILE = 'Sample.htm'
const EDITED = '<!doctype html><body><h1>Edited in Kition</h1></body>'
const OUTSIDE = '<!doctype html><body><h1>Edited in Kition</h1><p>Changed outside</p></body>'
const MORE = '<!doctype html><body><h1>Edited in Kition</h1><p>Changed outside</p><p>And here again</p></body>'
const server = await createServer({ configFile: new URL('../vite.config.ts', import.meta.url).pathname, server: { port: 0, host: '127.0.0.1' }, logLevel: 'error' })
await server.listen()
const origin = `http://127.0.0.1:${server.httpServer.address().port}`
const browser = await (which === 'webkit' ? webkit : chromium).launch()
const context = await browser.newContext({ viewport: { width: 1280, height: 800 } })
const page = await context.newPage()
context.setDefaultTimeout(10000)
const errors = []
let failures = 0
page.on('pageerror', (error) => errors.push(error.message))
const step = (name, ok, detail = '') => {
  if (!ok) failures += 1
  console.log(`${ok ? 'PASS' : 'FAIL'} ${name}${detail ? ` - ${detail}` : ''}`)
}
const until = async (check, tries = 100) => {
  for (let i = 0; i < tries; i += 1) {
    if (await check().catch(() => false)) return true
    await page.waitForTimeout(100)
  }
  return false
}
const kit = page.frameLocator('#kit')
const source = kit.getByTestId('html-source')
/** The engine's page source, as the editor's session holds it. */
const engineSource = () => page.evaluate(async () => {
  const answer = await window.devHost.connection().ai.inspect()
  return String((answer.details ?? answer).source ?? '')
})
/** The replica's own id, as the editor answers it. */
const docId = () => page.evaluate(() => window.devHost.connection().replica.info())
const fileVersion = () => page.evaluate((path) => window.devHost.files.get(path)?.version ?? 0, FILE)
const fileText = () => page.evaluate((path) => new TextDecoder().decode(window.devHost.files.get(path).bytes), FILE)
/** The stored replica's snapshot, or null. */
const storedSnapshot = async () => {
  const bytes = await page.evaluate((path) => {
    const stored = window.devHost.history.replica(path)
    return stored ? [...new Uint8Array(stored.snapshot)] : null
  }, FILE)
  return bytes ? new Uint8Array(bytes) : null
}
/** How many of the stored replica's changes are the system's `import` (a file taken in). */
const imports = (snapshot) => Replica.snapshotOrigins(snapshot).filter((origin) => origin?.kind === 'system' && origin.label === 'import').length
/** Waits for the editor in the frame to open the document again after `reopen` ran, showing `text`. */
const reopened = async (reopen, arg, text) => {
  const frame = await (await page.$('#kit')).contentFrame()
  await frame.evaluate(() => {
    window.beforeReopen = true
  })
  await page.evaluate(reopen, arg)
  const fresh = await (await page.$('#kit')).contentFrame()
  await fresh.waitForFunction(() => window.beforeReopen === undefined, null, { timeout: 60000 })
  await until(async () => (await source.isEnabled()) && (await source.inputValue()) === text, 300)
}
/** Saves through the editor (Ctrl+S in the source) and waits for the file and the stored replica. */
const save = async (text) => {
  const before = await fileVersion()
  await source.press('Control+s')
  return until(async () => (await fileVersion()) > before && (await fileText()) === text, 200)
}

try {
  await page.goto(`${origin}/dev.html?locale=en-US&replica=1&replica-merges-file=1`)
  await page.waitForFunction(() => document.getElementById('status')?.textContent?.includes(' open in '), null, { timeout: 90000 })
  await until(async () => source.isEnabled(), 300)
  step('the editor lists the replica capability', (await page.evaluate(() => window.devHost.connection().capabilities)).includes('replica'))
  const id = await docId()
  step('the page has a replica id', typeof id === 'string' && id.length > 0, String(id))

  // A person's edit, then a save: the file and the stored replica.
  await source.fill(EDITED)
  step('the edit reaches the engine', await until(async () => (await engineSource()) === EDITED))
  step('saves the file through the host', await save(EDITED), `file version ${await fileVersion()}`)
  step('the host stores the replica', await until(async () => (await storedSnapshot()) != null))
  const savedImports = imports(await storedSnapshot())

  // Reloaded as the save left it: the editor opens on the stored replica.
  await reopened(() => window.devHost.reload(), null, EDITED)
  step('the reloaded page has the edit', (await engineSource()) === EDITED, await engineSource())
  step('the reloaded page is the same replica', (await docId()) === id, `${await docId()} vs ${id}`)
  step('the stored replica is still kept after the reload', (await storedSnapshot()) != null)

  // Another program changes the file: the editor merges it into the stored replica.
  await reopened((text) => window.devHost.reopenChanged(new TextEncoder().encode(text).buffer), OUTSIDE, OUTSIDE)
  step('the outside change shows', (await engineSource()) === OUTSIDE, await engineSource())
  step('the page is still the same replica', (await docId()) === id, `${await docId()} vs ${id}`)
  step('the stored replica is still kept after the outside change', (await storedSnapshot()) != null)
  // An edit and a save store the replica as it is now: the outside change is one import on it.
  await source.fill(MORE)
  step('an edit after the outside change reaches the engine', await until(async () => (await engineSource()) === MORE))
  step('saves again', await save(MORE))
  step('the stored replica has the outside change as one import change', await until(async () => imports(await storedSnapshot()) === savedImports + 1), `${savedImports} imports before, ${imports(await storedSnapshot())} after`)

  // The saved file opened alone (no stored replica): the same page, as a replica of its own.
  await reopened((path) => window.devHost.open(path, window.devHost.files.get(path).bytes), FILE, MORE)
  step('the file alone has the page', (await engineSource()) === MORE, await engineSource())
  step('and is another replica', (await docId()) !== id, String(await docId()))

  step('no page errors', errors.length === 0, errors.join('; '))
} catch (error) {
  step('runs', false, error instanceof Error ? error.stack : String(error))
} finally {
  await browser.close()
  await server.close()
}
console.log(failures ? `${which}: ${failures} failed` : `${which}: all passed`)
process.exit(failures ? 1 : 0)
