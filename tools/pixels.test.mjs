import {test} from 'node:test';
import assert from 'node:assert/strict';
import {comparePixels} from './pixels.mjs';
test('counts even a one-level antialiasing difference without losing it in white space',()=>{
  const reference={width:2,height:1,data:Buffer.from([255,255,255,255,86,69,212,255])};
  const actual={width:2,height:1,data:Buffer.from([255,255,255,255,87,69,212,255])};
  const {metrics,diff}=comparePixels(reference,actual);
  assert.equal(metrics.changedPixels,1);
  assert.equal(metrics.significantPixels,0);
  assert.equal(metrics.foregroundChangedRatio,1);
  assert.equal(metrics.meanAbsoluteError,1/8);
  assert.deepEqual(metrics.differenceBounds,{x:1,y:0,width:1,height:1});
  assert.notDeepEqual([...diff.subarray(4)],[255,255,255,255]);
});
test('identity is exact and differing dimensions are errors',()=>{
  const image={width:1,height:1,data:Buffer.from([255,255,255,255])};
  assert.equal(comparePixels(image,image).metrics.changedPixels,0);
  assert.throws(()=>comparePixels(image,{...image,width:2}),/dimensions/);
});
