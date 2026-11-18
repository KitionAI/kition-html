/** Strict, unaligned RGBA comparison. No pixel or antialiasing exclusions. */
export function comparePixels(reference, actual) {
  if (reference.width !== actual.width || reference.height !== actual.height) {
    throw new Error('Render dimensions differ');
  }
  const {width, height} = reference;
  const count = width * height;
  const diff = Buffer.alloc(count * 4);
  const overlay = Buffer.alloc(count * 4);
  let changed = 0, significant = 0, foreground = 0, foregroundChanged = 0;
  let sum = 0, squares = 0, maxDelta = 0, maxChannelErrorSum = 0;
  let x0 = width, y0 = height, x1 = -1, y1 = -1;
  for (let pixel = 0; pixel < count; pixel++) {
    const at = pixel * 4;
    let delta = 0, active = false;
    for (let channel = 0; channel < 4; channel++) {
      const a = reference.data[at + channel], b = actual.data[at + channel];
      const d = Math.abs(a - b);
      delta = Math.max(delta, d);
      sum += d;
      squares += d * d;
      if (a !== 255 || b !== 255) active = true;
      overlay[at + channel] = channel === 3 ? 255 : Math.round((a + b) / 2);
    }
    if (active) foreground++;
    if (delta > 0) {
      changed++;
      if (active) foregroundChanged++;
      if (delta > 16) significant++;
      const x = pixel % width, y = Math.floor(pixel / width);
      x0 = Math.min(x0,x); y0 = Math.min(y0,y); x1 = Math.max(x1,x); y1 = Math.max(y1,y);
      // Every unequal pixel is visible, even a one-channel, one-level difference.
      diff[at] = 255;
      diff[at + 1] = 255 - Math.max(32,delta);
      diff[at + 2] = 255 - Math.max(32,delta);
    } else {
      diff[at] = diff[at + 1] = diff[at + 2] = 255;
    }
    diff[at + 3] = 255;
    maxDelta = Math.max(maxDelta,delta);
    maxChannelErrorSum += delta;
  }
  return {
    metrics: {pixels:count,changedPixels:changed,changedRatio:changed/count,
      significantPixels:significant,significantRatio:significant/count,
      foregroundPixels:foreground,foregroundChangedRatio:foreground ? foregroundChanged/foreground : 0,
      maxChannelErrorSum,similarity:1-maxChannelErrorSum/(count*255),
      foregroundSimilarity:foreground ? 1-maxChannelErrorSum/(foreground*255) : 1,
      meanAbsoluteError:sum/(count*4),rootMeanSquareError:Math.sqrt(squares/(count*4)),maxChannelDelta:maxDelta,
      differenceBounds:changed ? {x:x0,y:y0,width:x1-x0+1,height:y1-y0+1} : null},
    diff,overlay,
  };
}
