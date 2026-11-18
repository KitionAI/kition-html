// Verify the recorded behavioral oracle independently of rendered pixels.
// This command never rewrites expectations or comparison references.
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {chromium} from '@playwright/test';

const cases=JSON.parse(await readFile(new URL('../conformance/select-state-cases.json',import.meta.url),'utf8'));
const browser=await chromium.launch({headless:true});
try{
  const page=await browser.newPage();
  console.log(`Chrome ${browser.version()}: option state oracle`);
  for(const sample of cases){
    await page.setContent(`<!doctype html>${sample.body}`);
    const trace=await page.evaluate(script=>JSON.parse(eval(`${script};JSON.stringify(trace)`)),sample.script);
    assert.deepEqual(trace,sample.expected,sample.id);
    console.log(`PASS ${sample.id}`);
  }
  console.log(`${cases.length}/${cases.length} option state traces match Chrome`);
}finally{await browser.close();}
