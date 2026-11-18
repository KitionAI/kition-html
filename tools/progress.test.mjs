import {test} from 'node:test';
import assert from 'node:assert/strict';
import {compareRuns} from './progress.mjs';

const entry = (id, error, extra = {}) => ({id, sourceSha256: id, width: 640, height: 480, scale: 1, dark: false,
  metrics: {foregroundSimilarity: 1 - error / 255000, maxChannelErrorSum: error, foregroundPixels: 1000, pixels: 1000}, ...extra});

test('progress retains regressions and newly failing examples without diluting common-case counts', () => {
  const before = {cases: [entry('improves', 500), entry('regresses', 0), entry('same', 20), entry('removed', 0)]};
  const after = {cases: [entry('improves', 0), entry('regresses', 600), entry('same', 20), entry('new-failure', 100000)]};
  const result = compareRuns(before, after, Object.fromEntries(before.cases.map(entry => [entry.id, {before: 'same-png', after: 'same-png'}])));
  assert.deepEqual(result.summary, {improved: 1, regressed: 1, unchanged: 1, added: 1, removed: 1, incomparable: 0, 'render-error': 0, newPasses: 1, lostPasses: 1});
  assert.equal(result.cases.find(entry => entry.id === 'new-failure').afterPassed, false);
});

test('changed sources, viewports, or Chrome references cannot be counted as improvements', () => {
  const before = {cases: ['source', 'viewport', 'reference'].map(id => entry(id, 500))};
  const after = {cases: [entry('source', 0, {sourceSha256: 'changed'}), entry('viewport', 0, {scale: 2}), entry('reference', 0)]};
  const result = compareRuns(before, after, {reference: {before: 'old-png', after: 'new-png'}});
  assert.equal(result.summary.incomparable, 3);
  assert.equal(result.summary.improved, 0);
  assert.equal(result.summary.newPasses, 0);
});

test('render failures stay explicit even if a stale metric is present', () => {
  const result = compareRuns({cases: [entry('failed', 0)]}, {cases: [entry('failed', 0, {error: 'capture failed'})]});
  assert.equal(result.summary['render-error'], 1);
  assert.equal(result.summary.unchanged, 0);
});

test('missing reference hashes cannot establish renderer progress', () => {
  const result = compareRuns({cases: [entry('missing-reference', 500)]}, {cases: [entry('missing-reference', 0)]});
  assert.equal(result.summary.incomparable, 1);
  assert.equal(result.summary.improved, 0);
});
