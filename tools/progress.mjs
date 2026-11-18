import {readFile, writeFile} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {fileURLToPath} from 'node:url';
import path from 'node:path';
import {passesSimilarity} from './similarity.mjs';

export function compareRuns(before, after, references = {}) {
  const previous = new Map(before.cases.map(entry => [entry.id, entry]));
  const current = new Set(after.cases.map(entry => entry.id));
  const cases = after.cases.map(entry => {
    const old = previous.get(entry.id);
    const result = {id: entry.id, before: old?.metrics?.foregroundSimilarity,
      after: entry.metrics?.foregroundSimilarity,
      beforePassed: old?.metrics && !old.error ? passesSimilarity(old.metrics) : false,
      afterPassed: entry.metrics && !entry.error ? passesSimilarity(entry.metrics) : false};
    if (!old) return {...result, status: 'added'};
    if (!old.metrics || !entry.metrics || old.error || entry.error) return {...result, status: 'render-error'};
    const sameSource = typeof old.sourceSha256 === 'string' && old.sourceSha256.length > 0 && old.sourceSha256 === entry.sourceSha256;
    const sameProfile = ['width', 'height', 'scale', 'dark'].every(key => old[key] === entry[key]);
    const reference = references[entry.id];
    const sameReference = typeof reference?.before === 'string' && reference.before === reference.after;
    if (!sameSource || !sameProfile || !sameReference) {
      return {...result, status: 'incomparable', sameSource, sameProfile, sameReference};
    }
    const delta = result.after - result.before;
    return {...result, delta, status: delta > 0 ? 'improved' : delta < 0 ? 'regressed' : 'unchanged'};
  });
  for (const old of before.cases) {
    if (!current.has(old.id)) cases.push({id: old.id, before: old.metrics?.foregroundSimilarity, status: 'removed'});
  }
  const summary = Object.fromEntries(['improved', 'regressed', 'unchanged', 'added', 'removed', 'incomparable', 'render-error']
    .map(status => [status, cases.filter(entry => entry.status === status).length]));
  summary.newPasses = cases.filter(entry => entry.status === 'improved' && !entry.beforePassed && entry.afterPassed).length;
  summary.lostPasses = cases.filter(entry => entry.status === 'regressed' && entry.beforePassed && !entry.afterPassed).length;
  return {summary, cases};
}

async function main() {
  const [beforeId, afterId] = process.argv.slice(2);
  if (![beforeId, afterId].every(id => typeof id === 'string' && /^[a-zA-Z0-9_.-]+$/.test(id) && !['.', '..'].includes(id))) {
    throw Error('usage: node tools/progress.mjs <before-run> <after-run>');
  }
  const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
  const folder = id => path.join(root, 'artifacts', id);
  const load = async id => {
    // A checkpoint report is not a completed run.
    await readFile(path.join(folder(id), 'index.html'));
    return JSON.parse(await readFile(path.join(folder(id), 'report.json'), 'utf8'));
  };
  const before = await load(beforeId), after = await load(afterId);
  const oldIds = new Set(before.cases.map(entry => entry.id));
  const references = {};
  const digest = async file => createHash('sha256').update(await readFile(file)).digest('hex');
  for (const entry of after.cases) {
    if (!oldIds.has(entry.id) || entry.error) continue;
    if (!/^[a-zA-Z0-9_-]+$/.test(entry.id)) throw Error('Nonportable case identifier');
    try {
      references[entry.id] = {before: await digest(path.join(folder(beforeId), entry.id, 'chrome.png')),
        after: await digest(path.join(folder(afterId), entry.id, 'chrome.png'))};
    } catch { references[entry.id] = {before: 'unavailable-before', after: 'unavailable-after'}; }
  }
  const result = {before: beforeId, after: afterId, ...compareRuns(before, after, references)};
  const destination = path.join(folder(afterId), `progress-from-${beforeId}`);
  await writeFile(`${destination}.json`, JSON.stringify(result, null, 2) + '\n', {flag: 'wx'});
  const escape = value => String(value).replaceAll('&', '&amp;').replaceAll('<', '&lt;').replaceAll('>', '&gt;').replaceAll('"', '&quot;');
  const score = value => Number.isFinite(value) ? `${(value * 100).toFixed(6)}%` : 'Unavailable';
  const rows = [...result.cases].sort((a, b) => (a.delta ?? 0) - (b.delta ?? 0)).map(entry =>
    `<tr><th>${escape(entry.id)}</th><td>${entry.status}</td><td>${score(entry.before)}</td><td>${score(entry.after)}</td><td>${Number.isFinite(entry.delta) ? (entry.delta * 100).toFixed(6) : '—'}</td></tr>`).join('');
  await writeFile(`${destination}.html`, `<!doctype html><html lang="en"><meta charset="utf-8"><title>Kition HTML progress</title><style>body{font:14px/1.5 system-ui;margin:24px;background:#fafaf9;color:#1a1a1a}main{background:white;border:1px solid #e5e3df;border-radius:12px;padding:20px;overflow:auto}table{border-collapse:collapse;width:100%}td,th{text-align:left;padding:10px;border-bottom:1px solid #e5e3df}a{color:#5645d4}</style><main><h1>${escape(beforeId)} to ${escape(afterId)}</h1><p>Only cases with identical source hashes, device profiles, and Chrome PNG bytes count as renderer improvements or regressions. Changed references, additions, removals, and errors remain visible.</p><p>${Object.entries(result.summary).map(([key,value]) => `${key}: ${value}`).join(' · ')}</p><p><a href="index.html">Current comparison gallery</a></p><table><thead><tr><th>Case</th><th>Status</th><th>Before</th><th>After</th><th>Percentage point change</th></tr></thead><tbody>${rows}</tbody></table></main></html>`, {flag: 'wx'});
  console.log(JSON.stringify(result.summary, null, 2));
  console.log(`Report: artifacts/${afterId}/progress-from-${beforeId}.html`);
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) await main();
