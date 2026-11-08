// Focus continuity regression against a real Chromium DOM, independent
// of the unhydrated Dioxus HTML layout fixtures.
const assert = require('node:assert/strict');
const fs = require('node:fs/promises');
const path = require('node:path');

async function keyboardFocusChecks(browser) {
  const page = await browser.newPage();
  const script = await fs.readFile(path.join(__dirname, '../src/shortcuts/focus.js'), 'utf8');
  const render = async html => {
    await page.setContent('<!doctype html><html><body>' + html + '</body></html>');
    await page.addScriptTag({ content: '(async () => {\n' + script + '\n})();' });
  };
  try {
    await render('<main class="content" data-screen="Study">' +
      '<article class="st-item" tabindex="-1"><h1 tabindex="-1">Card</h1>' +
      '<button id="reveal" aria-keyshortcuts="Space">Reveal</button></article></main>');
    await page.evaluate(() => {
      document.querySelector('#reveal').addEventListener('click', () => {
        queueMicrotask(() => { document.querySelector('#reveal').outerHTML =
          '<div class="st-grades"><button id="good" class="rating" data-grade="good" aria-keyshortcuts="3">Good</button></div>'; });
      });
    });
    await page.locator('#reveal').focus();
    await page.evaluate(() => document.querySelector('#reveal').click());
    await page.waitForFunction(() => document.activeElement?.id === 'good');
    assert.equal(await page.evaluate(() => document.activeElement.id), 'good',
      'keyboard reveal should focus the available Good button');

    await page.evaluate(() => {
      document.querySelector('#good').addEventListener('click', () => {
        queueMicrotask(() => {
          document.querySelector('.st-item').outerHTML =
            '<article class="st-item" tabindex="-1"><h1 tabindex="-1">Next</h1><button id="next-reveal" aria-keyshortcuts="Space">Reveal next</button></article>';
        });
      });
    });
    await page.keyboard.press('Enter');
    await page.waitForFunction(() => document.activeElement?.id === 'next-reveal');
    assert.equal(await page.evaluate(() => document.activeElement.id), 'next-reveal',
      'keyboard grade should focus the next card');

    await render('<main class="content" data-screen="Study">' +
      '<article class="st-item" tabindex="-1"><button id="mouse" class="rating" data-grade="good">Good</button></article></main>');
    await page.evaluate(() => {
      document.querySelector('#mouse').addEventListener('click', () => {
        queueMicrotask(() => { document.querySelector('.st-item').outerHTML =
          '<article class="st-item" tabindex="-1"><button id="pointer-next" aria-keyshortcuts="Space">Next</button></article>'; });
      });
    });
    await page.locator('#mouse').click();
    await page.waitForSelector('#pointer-next');
    assert.notEqual(await page.evaluate(() => document.activeElement.id), 'pointer-next',
      'mouse grading must not force focus into the next card');

    await render('<main class="content" data-screen="Study">' +
      '<article class="st-item" tabindex="-1"><button id="modal-grade" class="rating" data-grade="good">Good</button></article>' +
      '</main><section role="dialog" aria-modal="true"><button id="modal-ok">OK</button></section>');
    await page.evaluate(() => {
      document.querySelector('#modal-grade').addEventListener('click', () => {
        document.querySelector('.st-item').outerHTML =
          '<article class="st-item" tabindex="-1"><button id="behind-modal" aria-keyshortcuts="Space">Next</button></article>';
      });
    });
    await page.locator('#modal-ok').focus();
    await page.evaluate(() => document.querySelector('#modal-grade').click());
    await page.waitForSelector('#behind-modal');
    assert.equal(await page.evaluate(() => document.activeElement.id), 'modal-ok',
      'modal must retain keyboard focus');

    await render('<main class="content" data-screen="Learn">' +
      '<div class="ls-player"><article class="ls-stage" tabindex="-1" data-n="1"><h2>Step 1</h2></article>' +
      '<article class="ls-stage" tabindex="-1" data-n="2" hidden><h2>Step 2</h2></article>' +
      '<footer class="ls-nav"><button id="next-step" data-nav="next" aria-keyshortcuts="Enter">Continue</button></footer></div></main>');
    await page.evaluate(() => {
      document.querySelector('#next-step').addEventListener('click', () => {
        queueMicrotask(() => {
          document.querySelector('.ls-stage[data-n="1"]').hidden = true;
          document.querySelector('.ls-stage[data-n="2"]').hidden = false;
        });
      });
    });
    await page.locator('#next-step').focus();
    await page.keyboard.press('Enter');
    await page.waitForFunction(() => document.activeElement?.matches('.ls-stage[data-n="2"]'));
    assert.equal(await page.evaluate(() => document.activeElement.dataset.n), '2',
      'keyboard lesson navigation should focus the active stage');

    await render('<main class="content" data-screen="Study">' +
      '<article class="st-item"><textarea id="typed">Clinical answer</textarea></article></main>');
    await page.locator('#typed').focus();
    await page.evaluate(() => {
      document.querySelector('.st-item').setAttribute('data-extra', 'update');
      document.querySelector('.st-item').append(document.createElement('span'));
    });
    assert.equal(await page.evaluate(() => document.activeElement.id), 'typed',
      'typing stays in the answer field');

    console.log('Keyboard focus continuity: reveal, grade, mouse, modal, lesson, and typing passed.');
  } finally {
    await page.close();
  }
}
module.exports = { keyboardFocusChecks };
