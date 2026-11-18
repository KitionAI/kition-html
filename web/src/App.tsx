import { useEffect, useMemo, useRef, useState, type ReactNode } from 'react'
import {
  Button, EditorFrame, FrameChrome, KitDocumentCanvas, KitEditorProvider, KitElement,
  catalogTabs, createKitStrings, useFrameMode, useKitEditor, useKitEditorState, useKitSession, useUi,
  type HostConnection, type KitCanvasHandle, type KitEngineInfo, type KitTool,
} from '@kition/kit-editor'
import { EDIT_TAB, HTML_MENUS, PRIMARY_ROWS } from './menus'

type HtmlState = { source: string }
const describe = (info: KitEngineInfo): HtmlState => ({ source: String(info.inspected.source ?? '') })
const catalogs = { 'en-US': { source: 'HTML source', preview: 'Preview', limits: 'HTML, CSS and JavaScript preview', noDiagnostics: 'No rendering diagnostics', previewFailed: 'Could not update preview', sourceFailed: 'Could not update HTML', fileChanged: 'This file changed on disk. Your unsaved edits have been kept.', keepMine: 'Keep my edits', reloadFile: 'Load saved version' } }

export function App({ host }: { host: HostConnection }) {
  const manifest = useMemo(() => ({ kitId: 'kit.html', base: new URL('./engine/', document.baseURI).href, module: 'kition_html_kit', documentClass: 'HtmlDocument' }), [])
  const base = useKitSession(host, manifest, describe)
  const pendingSource = useRef<Promise<unknown>>(Promise.resolve())
  const session = {
    ...base,
    save: async () => {
      await pendingSource.current
      return base.save()
    },
  }
  const t = useMemo(() => createKitStrings(catalogs, host.init.locale), [host])
  const editor = useKitEditorState(session, host, t, 'html')
  return <KitEditorProvider editor={editor}><HtmlEditor session={session} pendingSource={pendingSource} /></KitEditorProvider>
}

type Session = ReturnType<typeof useKitSession<HtmlState>>

function HtmlEditor({ session, pendingSource }: { session: Session; pendingSource: { current: Promise<unknown> } }) {
  const editor = useKitEditor()
  const ui = useUi()
  const canvas = useRef<KitCanvasHandle | null>(null)
  const [zoom, setZoom] = useState(1)
  const [source, setSource] = useState('')
  const [pending, setPending] = useState(0)
  const inFlight = useRef(0)
  const [diagnostics, setDiagnostics] = useState<string[]>([])
  const tabs = useMemo(() => catalogTabs(HTML_MENUS, { phrase: editor.phrase, primary: PRIMARY_ROWS }), [editor.phrase])
  const readOnly = editor.host.readOnly || ui.state('command:html.setSource', ['Edit']).denied

  // Local typing stays in the field until every queued command has reached the engine.
  // Agent edits, undo, and a clean on-disk reload then update it from the same session.
  useEffect(() => {
    if (pending === 0) setSource(session.state?.source ?? '')
  }, [session.state?.source, pending])

  useEffect(() => {
    let current = true
    if (session.doc && !pending) {
      void session.doc.check().then((items) => {
        if (current) setDiagnostics(items.map((item) => `${item.code}: ${item.message}`))
      }).catch((error: unknown) => { if (current) setDiagnostics([String(error)]) })
    }
    return () => { current = false }
  }, [session.doc, session.change.revision, pending])

  useEffect(() => {
    if (!session.doc) return
    let stopped = false
    let timer: ReturnType<typeof setTimeout>
    const tick = async () => {
      try {
        const result = await session.run('html.advance', { millis: 100 }, null, { refresh: 'none' })
        if (!stopped && result && JSON.parse(result).changed) {
          await canvas.current?.retryRender()
          const items = await session.doc!.check()
          if (!stopped) setDiagnostics(items.map((item) => `${item.code}: ${item.message}`))
        }
      } catch (error) {
        if (!stopped) setDiagnostics([String(error)])
        return
      }
      if (!stopped) timer = setTimeout(() => void tick(), 100)
    }
    timer = setTimeout(() => void tick(), 100)
    return () => { stopped = true; clearTimeout(timer) }
  }, [session.doc, session.run])

  const previewTool: KitTool = {
    id: 'Interact', cursor: 'default',
    begin: (start) => ({ end: (point) => {
      if (Math.hypot(point.clientX - start.clientX, point.clientY - start.clientY) > 4) return
      void session.run('html.click', { x: point.x, y: point.y }, null, { refresh: 'repaint' })
        .then(async () => {
          const items = await session.doc?.check()
          if (items) setDiagnostics(items.map((item) => `${item.code}: ${item.message}`))
        })
        .catch((error: unknown) => editor.report(editor.t('previewFailed'), error))
    } }),
  }

  const changeSource = (next: string) => {
    setSource(next)
    inFlight.current += 1
    setPending(inFlight.current)
    const applied = session.run('html.setSource', { source: next })
    pendingSource.current = applied
    void applied.catch((error: unknown) => {
      editor.report(editor.t('sourceFailed'), error)
    }).finally(() => {
      if (pendingSource.current === applied) pendingSource.current = Promise.resolve()
      inFlight.current -= 1
      setPending(inFlight.current)
    })
  }

  return (
    <EditorFrame tabs={tabs} defaultTab={EDIT_TAB} canvas={canvas} zoom={zoom} fileChangedBar={false} status={<span data-testid="html-status">{editor.t('limits')}</span>}>
      {session.conflict ? (
        <div className="html-conflict" role="alert" data-testid="html-conflict">
          <span>{editor.t('fileChanged')}</span>
          <Button onClick={() => void session.keepMine().catch((error: unknown) => editor.report(editor.t('saveFailed'), error))} data-testid="html-keep-mine">{editor.t('keepMine')}</Button>
          <Button onClick={() => session.reopen()} data-testid="html-reload">{editor.t('reloadFile')}</Button>
        </div>
      ) : null}
      <HtmlPanes>
        <KitElement id="panel:source">
          <section className="html-source-panel" aria-label={editor.t('source')}>
            <FrameChrome>
              <label className="html-panel-heading" htmlFor="html-source">{editor.t('source')}</label>
            </FrameChrome>
            <textarea id="html-source" data-testid="html-source" value={source} readOnly={readOnly} disabled={!session.doc || !editor.commands.length}
              spellCheck={false} autoCapitalize="off" autoCorrect="off" maxLength={1024 * 1024}
              onChange={(event) => changeSource(event.target.value)}
              onKeyDown={(event) => {
                if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 's') {
                  event.preventDefault()
                  if (!readOnly && !ui.state('action:save').denied) void editor.save()
                }
              }} />
          </section>
        </KitElement>
        <section className="html-preview-panel" aria-label={editor.t('preview')}>
          <FrameChrome>
            <div className="html-panel-heading">{editor.t('preview')}</div>
          </FrameChrome>
          <KitDocumentCanvas tool={previewTool} canvas={canvas} testId="html-preview" onView={(view) => setZoom(view.zoom)} />
          <FrameChrome>
            <div className="html-diagnostics" data-testid="html-diagnostics" role="status">
              {diagnostics.length ? diagnostics.map((item, index) => <div key={`${index}:${item}`}>{item}</div>) : editor.t('noDiagnostics')}
            </div>
          </FrameChrome>
        </section>
      </HtmlPanes>
    </EditorFrame>
  )
}

/**
 * The source and the preview side by side. In Focus only they show (their headings and the
 * diagnostics are the frame's chrome), ending above the Focus bar, which would cover the source.
 */
function HtmlPanes({ children }: { children: ReactNode }) {
  const focus = useFrameMode() === 'focus'
  return <div className={focus ? 'html-editor html-editor-focus' : 'html-editor'}>{children}</div>
}
