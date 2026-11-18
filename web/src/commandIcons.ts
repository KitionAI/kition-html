/**
 * Tessera icons for the HTML engine's commands that kit-ui's rules read wrongly: here "source" is
 * the page's HTML, not a data source. Registered once, when the editor loads (`main.tsx`).
 */
import { registerCommandIcons, TesseraIcons } from '@kition/kit-editor'

registerCommandIcons({
  'html.setSource': TesseraIcons.CodeIcon,
})
