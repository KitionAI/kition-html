// An agent's draft and a restored version in the editor (Kit UI protocol sections 7d and 7m), through
// the development host with `?replica=1&checkpoints=1`: node tests/drafts.mjs [chromium|webkit].
// The page is one text field (`Page.source`), so a draft and an edit of the person made while the
// draft is open always collide (data model 6.6, D5: a text both changed collides, whatever the
// ranges). Covered: (a) the person edits, the agent's draft makes another edit, its comparison lists
// it and leaves the page alone, it is accepted and both edits show in the source the editor shows;
// a person's edit made while the draft is open makes the merge refuse it, naming the collision, and
// the page stays as it was; (b) one Undo of the editor takes back the draft's edit only and Redo
// brings it back; (d) a labelled checkpoint, a later edit, the checkpoint restored through the host,
// and one Undo bringing the later edit back.
// Needs the engine and fonts built into public/ (pnpm run engine). Run it with no HTTP proxy set.
import assert from 'node:assert/strict'
import { chromium, webkit } from '@playwright/test'
import { createServer } from 'vite'

const which = process.argv[2] ?? 'chromium'
const FILE = 'Sample.htm'
const server = await createServer({ configFile: new URL('../vite.config.ts', import.meta.url).pathname, server: { port: 0, host: '127.0.0.1' }, logLevel: 'error' })
await server.listen()
const browser = await (which === 'webkit' ? webkit : chromium).launch()
const page = await browser.newPage({ viewport: { width: 1280, height: 800 } })
const kit = page.frameLocator('#kit')
const source = kit.getByTestId('html-source')
const errors = []
page.on('pageerror', (error) => errors.push(error.message))
const until = async (check, message) => {
  for (let n = 0; n < 300; n += 1) {
    if (await check().catch(() => false)) return
    await page.waitForTimeout(100)
  }
  assert.fail(message)
}
/** The page's source as the engine answers it, in the document or (`draft`) in the agent's draft. */
const engineSource = (draft = false) => page.evaluate(async (draft) => {
  const answer = await window.devHost.connection().ai.inspect(draft ? { draft: true } : {})
  return String((answer.details ?? answer).source ?? '')
}, draft)
/** The source the editor shows and the engine holds are both `text`. */
const shows = (text) => async () => (await source.inputValue()) === text && (await engineSource()) === text
/** The person types the whole source `text` into the editor's source field. */
const type = async (text) => {
  await source.fill(text)
  await until(shows(text), `the person's edit reaches the engine: ${text.slice(0, 80)}`)
}
/** The agent sets the source in its draft (the draft opens with its first edit). */
const draftEdit = (text) => page.evaluate(async (text) => {
  const result = await window.devHost.connection().ai.run([{ command: 'file.setSource', params: { source: text } }], false, { draft: true })
  return result.completed.length
}, text)
const press = async (id) => {
  await until(() => kit.getByTestId(id).isEnabled(), `${id} is enabled`)
  await kit.getByTestId(id).click()
}
const fileVersion = () => page.evaluate((path) => window.devHost.files.get(path)?.version ?? 0, FILE)
let failures = 0

try {
  await page.goto(`http://127.0.0.1:${server.httpServer.address().port}/dev.html?locale=en-US&replica=1&checkpoints=1`)
  await page.waitForFunction(() => document.getElementById('status')?.textContent?.includes(' open in '), null, { timeout: 90000 })
  await until(() => source.isEnabled(), 'the source field is enabled')
  const capabilities = await page.evaluate(() => window.devHost.connection().capabilities)
  assert.ok(capabilities.includes('draft'), `the editor lists the draft capability (${capabilities})`)
  const sample = await source.inputValue()
  assert.ok(sample.includes('<h1>Spring concert</h1>') && sample.includes('Doors open at 7.30 pm.'), 'the sample is open')

  // (a) The person edits; the agent's draft makes another edit, compared without changing the page.
  const personal = sample.replace('<h1>Spring concert</h1>', '<h1>Spring concert 2026</h1>')
  await type(personal)
  const drafted = personal.replace('Doors open at 7.30 pm.', 'Doors open at 7 pm.')
  assert.equal(await draftEdit(drafted), 1, 'the draft takes the agent\'s edit')
  assert.equal(await engineSource(true), drafted, 'the draft holds the agent\'s edit')
  const compared = await page.evaluate(() => window.devHost.connection().draft.compare({ maxSize: 256 }))
  const listed = [...compared.summary.changed, ...compared.summary.created].some((change) => change.fields.includes('source'))
    || compared.summary.texts.some((text) => text.field === 'source' && text.after.includes('Doors open at 7 pm.'))
  assert.ok(listed, `the comparison lists the draft's edit: ${JSON.stringify(compared.summary)}`)
  assert.ok(await shows(personal)(), 'the comparison changes nothing in the page')
  const merged = await page.evaluate(() => window.devHost.connection().draft.merge({ kind: 'agent', principal: 'agent:test' }))
  assert.equal(merged.merged, true, 'the draft merged')
  await until(shows(drafted), 'the editor shows the person\'s edit and the draft\'s')
  assert.ok((await source.inputValue()).includes('<h1>Spring concert 2026</h1>'), 'the person\'s edit stays')
  console.log(`PASS ${which}: a draft compared and accepted keeps the person's edit (a)`)

  // (b) One Undo of the editor takes back the draft's edit only; Redo brings it back.
  await press('html-undo')
  await until(shows(personal), 'Undo takes back the draft\'s edit, not the person\'s')
  await press('html-redo')
  await until(shows(drafted), 'Redo brings the draft\'s edit back')
  console.log(`PASS ${which}: one Undo takes back the draft only, Redo brings it back (b)`)

  // A person's edit while a draft is open: the page's one text both changed, the merge refuses it.
  const agentOnly = drafted.replace('Saturday 8 pm', 'Saturday 8.30 pm')
  await draftEdit(agentOnly)
  const meanwhile = drafted.replace('Book a ticket', 'Book now')
  await type(meanwhile)
  const refused = await page.evaluate(() => window.devHost.connection().draft.merge({ kind: 'agent', principal: 'agent:test' }).then(() => null, (error) => String(error?.message ?? error)))
  assert.ok(refused && /collide/i.test(refused), `the merge is refused as a collision: ${refused}`)
  await until(shows(meanwhile), 'the refused draft leaves the page as the person left it')
  await page.evaluate(() => window.devHost.connection().draft.discard())
  console.log(`PASS ${which}: a draft colliding with the person's edit is refused (${refused.slice(0, 120)})`)

  // (d) A labelled checkpoint of the saved page, a later edit, the checkpoint restored, then Undo.
  const before = await fileVersion()
  await source.press('Control+s')
  await until(async () => (await fileVersion()) > before, 'saved')
  await until(() => page.evaluate((path) => window.devHost.history.replica(path) !== null, FILE), 'the replica is stored with the save')
  const checkpoint = await page.evaluate(() => window.devHost.recordCheckpoint({ origin: 'save', label: 'Before the later edit' }))
  assert.ok(checkpoint?.id, 'a checkpoint is recorded')
  const labels = await page.evaluate((path) => window.devHost.history.checkpoints(path).map((item) => item.label), FILE)
  assert.ok(labels.includes('Before the later edit'), `the checkpoint is listed with its label: ${JSON.stringify(labels)}`)
  const later = meanwhile.replace('Tickets left:', 'Seats left:')
  await type(later)
  await page.evaluate((id) => window.devHost.restoreCheckpoint(id), checkpoint.id)
  await until(shows(meanwhile), 'the restore gives back the checkpoint\'s page')
  await press('html-undo')
  await until(shows(later), 'one Undo takes the restore back')
  console.log(`PASS ${which}: a restored checkpoint is one step of the person's Undo (d)`)
  assert.deepEqual(errors, [], 'no page errors')
} catch (error) {
  failures += 1
  console.log(`FAIL ${which}: ${error instanceof Error ? error.message.split('\n')[0] : String(error)}`)
} finally {
  if (errors.length) console.log(`page errors:\n${errors.slice(0, 8).join('\n')}`)
  await browser.close()
  await server.close()
}
process.exit(failures ? 1 : 0)
