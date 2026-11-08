// Targeted CSS hit-area regression for compact controls outside the
// lesson/study SSR. Uses production CSS, but not hydrated bank/calc actions.
const assert = require('node:assert/strict');
const fs = require('node:fs/promises');
const path = require('node:path');
const { pathToFileURL } = require('node:url');

async function mobileControlChecks(page, fixtures) {
  const study = path.join(fixtures, 'study');
  const manifest = JSON.parse(await fs.readFile(path.join(study, 'manifest.json'), 'utf8'));
  const setup = manifest.find(item => item.phase === 'setup');
  assert.ok(setup, 'study setup fixture exists');
  await page.goto(pathToFileURL(path.join(study, setup.file)).href);
  await page.addStyleTag({ path: path.join(__dirname, '../src/calc/calc.css') });
  for (const size of ['normal', 'larger']) {
    await page.evaluate(size => new Promise(resolve => {
      const link = document.querySelector('#prefs');
      const href = `prefs-system-${size}.css`;
      if (link.getAttribute('href') === href) return resolve();
      link.onload = resolve;
      link.setAttribute('href', href);
    }), size);
    for (const width of [320, 384]) {
      await page.setViewportSize({ width, height: 900 });
      const controls = await page.evaluate(() => {
        const host = document.createElement('section');
        host.className = 'card';
        host.innerHTML =
          '<div class="view-bar"><div class="view-sort"><button class="view-sort-btn">ترتيب</button></div>' +
          '<div class="view-seg"><button aria-label="Cards">C</button><button aria-label="List">L</button>' +
          '<button aria-label="Compact">S</button></div></div>' +
          '<div class="calc-form"><div class="calc-foot"><button class="btn-link">إعادة</button></div>' +
          '<div class="calc-result"><details class="calc-parts"><summary>التفاصيل</summary></details></div></div>' +
          '<div class="section-head"><button class="btn-ghost">عرض الكل</button></div>';
        document.querySelector('main.content').append(host);
        const result = [...host.querySelectorAll('button, summary')].map(el => {
          const box = el.getBoundingClientRect();
          return { name: el.getAttribute('aria-label') || el.textContent.trim(),
            width: box.width, height: box.height, left: box.left, right: box.right };
        });
        host.remove();
        return result;
      });
      assert.equal(controls.length, 7);
      assert.deepEqual(controls.filter(x =>
        x.width < 23.5 || x.height < 47.5 || x.left < -1 || x.right > width + 1), [],
      `compact controls at ${width}px (${size}): ${JSON.stringify(controls)}`);
    }
  }
  console.log('Compact list/calculator controls: >=48dp at 320/384px.');
}
module.exports = { mobileControlChecks };
