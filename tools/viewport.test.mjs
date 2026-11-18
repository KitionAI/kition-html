import {test} from 'node:test';
import assert from 'node:assert/strict';
import {captureCases, viewportDescription} from './viewport.mjs';

const fixture = Object.freeze({id:'responsive-fixture',width:640,height:480,scale:1,dark:true});

test('device profiles preserve source fixtures and describe actual capture dimensions', () => {
  const baseline = captureCases([fixture], []);
  const dense = captureCases([fixture], ['--scale','2']);
  const narrow = captureCases([fixture], ['--width','320','--height','600','--scale','1.5']);
  assert.deepEqual(baseline, [fixture]);
  assert.equal(dense[0].width, 640);
  assert.equal(dense[0].scale, 2);
  assert.equal(narrow[0].dark, true);
  assert.equal(fixture.scale, 1);
  assert.equal(viewportDescription([...dense,...narrow]),
    '640 x 480 CSS pixels / DPR 2; 320 x 600 CSS pixels / DPR 1.5');
});

test('invalid or excessive captures fail before browser or native rendering', () => {
  for (const args of [
    ['--scale'], ['--scale','NaN'], ['--scale','0'], ['--scale','5'],
    ['--width','1.5'], ['--height','-1'], ['--width','Infinity'],
    ['--width','16385'], ['--width','4096','--height','4097'],
    ['--scale','0.0001'], ['--width','4294967296','--scale','0.000001'],
  ]) assert.throws(() => captureCases([fixture], args));
  assert.doesNotThrow(() => captureCases([fixture], ['--width','4096','--height','4096']));
});
