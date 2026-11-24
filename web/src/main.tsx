import { createRoot } from 'react-dom/client'
import { connectHost, defineKitUi } from '@kition/sdk/editor'
import '@kition/sdk/editor/styles.css'
import './app.css'
import { App } from './App'
import { HTML_MENUS } from './menus'
import manifest from '../../kit.json'
import './commandIcons'

const root = createRoot(document.getElementById('root')!)
const font = new FontFace('Inter', `url(${new URL('./fonts/Inter-Variable.ttf', document.baseURI).href})`, { weight: '100 900' })
document.fonts.add(font)
void font.load().catch(() => undefined)
const alone = window.setTimeout(() => root.render(<p className="kui-message">Open an HTML file from your workspace to use this editor.</p>), 3000)
void connectHost({
  id: manifest.id, version: manifest.version, capabilities: ['ai', 'replica'], brand: manifest.ui.brand, display: manifest.ui.display,
  ui: defineKitUi({ menus: HTML_MENUS, tools: [], panels: [{ id: 'panel:source', label: 'HTML source' }], tabs: [], groups: [], controls: [] }),
}).then((host) => {
  window.clearTimeout(alone)
  root.render(<App host={host} />)
}).catch((error: unknown) => {
  window.clearTimeout(alone)
  root.render(<p role="alert">{String(error)}</p>)
})
