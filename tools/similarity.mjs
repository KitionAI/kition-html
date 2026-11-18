/** Versioned, per-case acceptance policy; exact pixel metrics remain diagnostic. */
export const similarityPolicy = Object.freeze({
  version: 1,
  threshold: 0.999,
  metric: '1 - sum(max(abs(referenceRGBA - actualRGBA))) / (255 * foregroundPixels)',
  foreground: 'union of pixels other than opaque white in either image',
  scope: 'every case independently; render errors fail; no exclusions or alignment',
});

export function passesSimilarity(metrics) {
  if (!metrics || !Number.isSafeInteger(metrics.maxChannelErrorSum)
      || metrics.maxChannelErrorSum < 0 || !Number.isSafeInteger(metrics.foregroundPixels)
      || metrics.foregroundPixels < 0 || !Number.isSafeInteger(metrics.pixels)
      || metrics.pixels <= 0 || metrics.foregroundPixels > metrics.pixels) return false;
  // Integer arithmetic keeps a score displayed as 99.900% from deciding the gate.
  return metrics.maxChannelErrorSum * 1000 <= metrics.foregroundPixels * 255;
}

export function summarizeSimilarity(metrics) {
  const passed = metrics.filter(passesSimilarity).length;
  return {total: metrics.length, passed, failed: metrics.length - passed,
    minimumForegroundSimilarity: metrics.length && metrics.every(m => Number.isFinite(m?.foregroundSimilarity))
      ? Math.min(...metrics.map(m => m.foregroundSimilarity)) : null};
}
