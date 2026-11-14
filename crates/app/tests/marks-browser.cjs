// Marks on a lesson picture (#984, learn/steps/marks.rs) in a real
// Chromium: every mark lands on its spot (a fraction of the picture) at a
// phone and a Mac width, in RTL and LTR, with a short window that caps the
// picture's height, and still after the full-screen viewer zooms and pans
// (viewer.js). Hidden marks draw nothing; the toggles are 48dp, named and
// keyed. Fixtures: learn/browser.rs `export_screen_fixtures_marks`.
const assert = require('node:assert/strict');
const fs = require('node:fs/promises');
const path = require('node:path');
const { pathToFileURL } = require('node:url');

const WIDTHS = [384, 1280];
const HEIGHTS = [900, 500];
const DIRS = ['rtl', 'ltr'];
// A mark's spot may be off by this much (sub-pixel rounding, borders).
const SLACK = 1.5;

// Where each mark is against where its fractions put it, in the picture
// that `scope` holds. Runs in the page.
function measureMarks([scope, marks]) {
  const root = document.querySelector(scope);
  const img = root.querySelector('.ls-marked img').getBoundingClientRect();
  const out = [];
  const els = [...root.querySelectorAll('.ls-marked .ls-mk')];
  els.forEach((el, i) => {
    const m = marks[i];
    const r = el.getBoundingClientRect();
    const want = { x: img.left + m.at[0] * img.width, y: img.top + m.at[1] * img.height };
    const got = { x: r.left + r.width / 2, y: r.top + r.height / 2 };
    const off = Math.max(Math.abs(got.x - want.x), Math.abs(got.y - want.y));
    let size = 0;
    if (m.size) {
      size = Math.max(Math.abs(r.width - m.size[0] * img.width), Math.abs(r.height - m.size[1] * img.height));
    }
    const n = el.querySelector('.ls-mk-n').getBoundingClientRect();
    const clipped = Math.max(img.left - n.left, n.right - img.right, img.top - n.top, n.bottom - img.bottom);
    out.push({ i: i + 1, kind: m.kind, off, size, clipped, number: el.querySelector('.ls-mk-n').textContent });
  });
  return { count: els.length, img: { w: img.width, h: img.height }, out };
}

function controls() {
  const main = document.querySelector('main.content');
  const shown = el => el.getClientRects().length > 0 && !el.closest('[hidden], [inert]');
  const all = [...document.querySelectorAll('.ls-figure button, .ls-viewer button')].filter(shown);
  const toggles = [...document.querySelectorAll('.ls-marks-toggle')].filter(shown).map(el => {
    const r = el.getBoundingClientRect();
    return { w: r.width, h: r.height, left: r.left, right: r.right, pressed: el.getAttribute('aria-pressed'),
      key: el.getAttribute('aria-keyshortcuts'), name: (el.getAttribute('aria-label') || el.textContent).trim() };
  });
  const list = [...document.querySelectorAll('.ls-marks-list')].filter(shown).map(ol => ({
    label: ol.getAttribute('aria-label'),
    items: [...ol.querySelectorAll('li')].map(li => li.querySelector('bdi')?.getAttribute('dir')),
  }));
  return {
    documentWidth: document.documentElement.scrollWidth,
    mainOverflow: main.scrollWidth - main.clientWidth,
    small: all.filter(el => el.getBoundingClientRect().height < 47.5).map(el => el.className),
    offscreen: all.filter(el => {
      const r = el.getBoundingClientRect();
      return r.left < -1 || r.right > innerWidth + 1;
    }).map(el => el.className),
    toggles,
    list,
    marks: [...document.querySelectorAll('.ls-mk')].filter(shown).length,
  };
}

function check(name, r, marks) {
  assert.equal(r.count, marks.length, `${name}: one drawn mark per mark`);
  assert.ok(r.img.w > 50 && r.img.h > 50, `${name}: the picture has a size ${JSON.stringify(r.img)}`);
  for (const m of r.out) {
    assert.ok(m.off <= SLACK, `${name}: mark ${m.i} (${m.kind}) is ${m.off.toFixed(1)}px off its spot`);
    assert.ok(m.size <= SLACK + 0.5, `${name}: mark ${m.i} (${m.kind}) is ${m.size.toFixed(1)}px off its size`);
    assert.ok(m.clipped <= 1, `${name}: mark ${m.i}'s number runs ${m.clipped.toFixed(1)}px past the picture`);
    assert.equal(m.number, String(m.i), `${name}: mark ${m.i} is numbered`);
  }
}

async function setDir(page, dir) {
  await page.evaluate(dir => {
    document.documentElement.dir = dir;
    document.querySelector('.app').dir = dir;
  }, dir);
}

async function marksChecks(page, fixtures, output) {
  const dir = path.join(fixtures, 'marks');
  const manifest = JSON.parse(await fs.readFile(path.join(dir, 'manifest.json'), 'utf8'));
  const viewer = await fs.readFile(path.join(__dirname, '../src/learn/viewer.js'), 'utf8');
  let cases = 0;
  for (const item of manifest) {
    const marks = item.marks;
    for (const direction of DIRS) {
      for (const width of WIDTHS) {
        for (const height of HEIGHTS) {
          const name = `${item.file.replace(/\.html$/, '')}-${direction}-${width}x${height}`;
          await page.setViewportSize({ width, height });
          await page.goto(pathToFileURL(path.join(dir, item.file)).href);
          await page.evaluate(() => Promise.all([document.fonts.ready,
            ...[...document.images].map(i => i.complete ? null : new Promise(r => { i.onload = r; i.onerror = r; }))]));
          await setDir(page, direction);
          const c = await page.evaluate(controls);
          assert.ok(c.documentWidth <= width + 1 && c.mainOverflow <= 1, `${name}: wider than the screen ${JSON.stringify(c)}`);
          assert.deepEqual(c.small, [], `${name}: figure controls under 48dp`);
          assert.deepEqual(c.offscreen, [], `${name}: figure controls off the screen`);
          for (const t of c.toggles) {
            assert.ok(t.w >= 47.5 && t.h >= 47.5, `${name}: the marks toggle is 48dp ${JSON.stringify(t)}`);
            assert.equal(t.key, 'M', `${name}: the marks toggle has its key`);
            assert.ok(t.name, `${name}: the marks toggle is named`);
            assert.equal(t.pressed, String(item.shown), `${name}: the toggle says whether marks show`);
          }
          if (!item.shown) {
            assert.equal(c.marks, 0, `${name}: hidden marks draw nothing`);
            assert.deepEqual(c.list, [], `${name}: hidden marks list nothing`);
            cases++;
            continue;
          }
          for (const l of c.list) {
            assert.ok(l.label, `${name}: the labels list is named`);
            assert.deepEqual(l.items, marks.map(() => 'ltr'), `${name}: one English label per mark`);
          }
          if (!item.zoomed) {
            check(name, await page.evaluate(measureMarks, ['.ls-figure', marks]), marks);
            await page.evaluate(() => document.querySelector('.ls-figure').scrollIntoView());
            if (direction === 'rtl' && height === 900) {
              await page.locator('.ls-figure').screenshot({ path: path.join(output, `${name}.png`) }).catch(() => {});
            }
          } else {
            await page.evaluate(src => (0, eval)(src), viewer);
            check(`${name}-fit`, await page.evaluate(measureMarks, ['.ls-viewer', marks]), marks);
            for (const zoom of ['in', 'in', 'in']) {
              await page.click(`.ls-viewer [data-zoom="${zoom}"]`);
            }
            const scale = await page.evaluate(() => +document.querySelector('.ls-viewer').dataset.scale);
            assert.ok(scale > 3, `${name}: zoomed in (${scale})`);
            check(`${name}-zoomed`, await page.evaluate(measureMarks, ['.ls-viewer', marks]), marks);
            // Pan to the far corner: the marks move with the picture.
            await page.evaluate(() => {
              const sc = document.querySelector('.ls-viewer-scroll');
              sc.scrollLeft = sc.scrollWidth;
              sc.scrollTop = sc.scrollHeight;
            });
            check(`${name}-panned`, await page.evaluate(measureMarks, ['.ls-viewer', marks]), marks);
            await page.click('.ls-viewer [data-zoom="fit"]');
            check(`${name}-back`, await page.evaluate(measureMarks, ['.ls-viewer', marks]), marks);
          }
          cases++;
        }
      }
    }
  }
  console.log(`Checked marks on a lesson picture in ${cases} cases.`);
}

module.exports = { marksChecks };
