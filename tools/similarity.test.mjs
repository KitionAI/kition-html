import {test} from 'node:test';
import assert from 'node:assert/strict';
import {comparePixels} from './pixels.mjs';
import {passesSimilarity,summarizeSimilarity} from './similarity.mjs';

const image = (width, data) => ({width,height:1,data:Buffer.from(data)});

test('accepts isolated rounding errors while retaining exact diagnostics',()=>{
  const a=image(10,Array(10).fill([86,69,212,255]).flat());
  const b={...a,data:Buffer.from(a.data)};
  b.data[0]++;
  const {metrics}=comparePixels(a,b);
  assert.equal(metrics.changedPixels,1);
  assert.equal(metrics.maxChannelErrorSum,1);
  assert.equal(metrics.foregroundSimilarity,1-1/2550);
  assert.equal(passesSimilarity(metrics),true);
});

test('white padding cannot hide a missing small element',()=>{
  const a=image(2000,Array(2000).fill([255,255,255,255]).flat());
  const b={...a,data:Buffer.from(a.data)};
  a.data[0]=a.data[1]=a.data[2]=0;
  const {metrics}=comparePixels(a,b);
  assert.equal(metrics.similarity,0.9995);
  assert.equal(metrics.foregroundSimilarity,0);
  assert.equal(passesSimilarity(metrics),false);
});

test('99.9 percent is inclusive and an extra unit of error fails before rounding',()=>{
  const metrics={pixels:1000,foregroundPixels:1000,maxChannelErrorSum:255};
  assert.equal(passesSimilarity(metrics),true);
  assert.equal(passesSimilarity({...metrics,maxChannelErrorSum:256}),false);
  assert.equal(passesSimilarity(null),false);
  assert.equal(passesSimilarity({...metrics,maxChannelErrorSum:NaN}),false);
  assert.equal(summarizeSimilarity([metrics,null]).failed,1);
});

test('white identity passes, alpha and extra content remain measurable',()=>{
  const a=image(1,[255,255,255,255]);
  assert.equal(passesSimilarity(comparePixels(a,a).metrics),true);
  const b=image(1,[255,255,255,254]);
  const {metrics}=comparePixels(a,b);
  assert.equal(metrics.foregroundPixels,1);
  assert.equal(metrics.maxChannelErrorSum,1);
  assert.equal(passesSimilarity(metrics),false);
});
