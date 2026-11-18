// Capture overrides keep the fixture corpus immutable across device profiles.
export function captureCases(cases, args) {
  const overrides = {};
  for (const key of ['width', 'height', 'scale']) {
    const index = args.indexOf(`--${key}`);
    if (index < 0) continue;
    const value = Number(args[index + 1]);
    if (!Number.isFinite(value) || value <= 0
        || (key === 'scale' ? value > 4 : !Number.isSafeInteger(value) || value > 4294967295)) {
      throw new Error(`Invalid --${key}: expected ${key === 'scale' ? '0 < scale <= 4' : 'a positive integer'}`);
    }
    overrides[key] = value;
  }
  return cases.map(test => {
    const result = {...test, ...overrides};
    const width = Math.round(result.width * result.scale);
    const height = Math.round(result.height * result.scale);
    if (width < 1 || height < 1 || width > 16384 || height > 16384
        || width * height > 16777216) {
      throw new Error(`Capture ${test.id} exceeds the engine raster limits`);
    }
    return result;
  });
}

export function viewportDescription(cases) {
  return [...new Set(cases.map(({width, height, scale}) =>
    `${width} x ${height} CSS pixels / DPR ${scale}`))].join('; ');
}
