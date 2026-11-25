/**
 * The HTML editor's menus: one catalog of rows (the engine's Edit commands), drawn by kit-editor's
 * `EditorFrame` in every display mode. Declared here, not built from the engine's command list, so
 * the frame has its Edit tab from the first drawing (it would show Menus until the engine answered).
 */
import type { MenuRow } from '@kition/sdk/editor'

const EDIT = ['Edit']

export const HTML_MENUS: readonly MenuRow[] = [
  { path: EDIT, label: 'Replace HTML…', command: 'file.setSource' },
  { path: EDIT, label: '---', command: '---' },
  { path: EDIT, label: 'Undo', command: 'edit.undo' },
  { path: EDIT, label: 'Redo', command: 'edit.redo' },
]

/** The tab the editor opens on. */
export const EDIT_TAB = 'Edit'

/** Single line's row: every Edit row (the engine has these three commands). */
export const PRIMARY_ROWS: Readonly<Record<string, readonly string[]>> = {
  Edit: ['file.setSource', 'edit.undo', 'edit.redo'],
}
