// Browser layout checks of HTML rendered from the production lesson player
// and study screen (learn/browser.rs, study/browser.rs) with the app's own
// stylesheets. No Dioxus hydration: taps, saving, grading, native Back and
// device QA are separate. Usage: screens-browser.cjs <fixtures> <results>
const assert = require('node:assert/strict');
const fs = require('node:fs/promises');
const path = require('node:path');
const { pathToFileURL } = require('node:url');
const { chromium } = require('playwright');
const { keyboardFocusChecks } = require('./keyboard-focus-browser.cjs');
const { highlightChecks } = require('./highlight-browser.cjs');
// The compact controls check also verifies bank scope/history touch targets.
const { mobileControlChecks } = require('./mobile-touch-browser.cjs');

const WIDTHS = [320, 384, 600, 840, 1280, 1600];
const SCHEMES = ['light', 'dark'];
const APPEARANCES = ['calm', 'glossy'];
const SIZES = ['normal', 'larger'];
// The floor each screen keeps: the lesson player promises the shared 48dp
// `--target` (assets/parts.css) for every control; the study screen sizes
// its buttons with the app's `--tap`/`--tap-lg` tokens (assets/main.css), so
// it is held to `--tap` (at most 48px) here and its controls under 48px are counted in the
// report as a known gap.
const FLOOR = { learn: 'var(--target)', study: 'min(var(--tap), var(--target))' };

// Gaps on main when this suite was added (2026-10-30). They are left out of
// the failures, listed in report.json and the log so they stay visible, and
// each is a separate fix (docs/SCREENS-BROWSER-QA.md). Remove an entry with
// its fix; an entry nothing matches any more is reported as stale.
const KNOWN = {
  // [screen, selector]: controls still under the screen's floor.
  small: [],
  // States whose controls or content are wider than the screen.
  overflow: [],
  // Running study items have no heading for screen readers to jump to.
  noHeading: [],
};

// The lesson's reading column, measured in the page (learn.css ≥840px).
function readingColumn() {
  const player = document.querySelector('.ls-player[data-view="steps"]');
  const main = document.querySelector('main.content');
  const context = player.querySelector(':scope > .ls-context').getBoundingClientRect();
  const stage = player.querySelector(':scope > .ls-stage:not([hidden])').getBoundingClientRect();
  const nav = player.querySelector(':scope > .ls-nav').getBoundingClientRect();
  const m = main.getBoundingClientRect();
  const style = getComputedStyle(main);
  const inner = (m.left + parseFloat(style.paddingLeft) + m.right - parseFloat(style.paddingRight)) / 2;
  return { stageWidth: stage.width, stageTop: stage.top, contextBottom: context.bottom,
    // A long step scrolls under Back/Next, which then stick to the bottom.
    navGap: nav.top - stage.bottom, navStuck: nav.bottom >= m.bottom - parseFloat(style.paddingBottom) - 2, navBottom: nav.bottom, mainBottom: m.bottom, offCentre: (stage.left + stage.right) / 2 - inner };
}

// Everything a finger or the keyboard can reach on screen, with the layout
// facts the assertions need. Runs in the page.
function measure([floor, known]) {
  const probe = document.createElement('div');
  probe.style.blockSize = floor;
  document.querySelector('.app').append(probe);
  const target = probe.getBoundingClientRect().height;
  probe.remove();
  const shown = el => el.getClientRects().length > 0 && !el.closest('[hidden], [inert]') &&
    getComputedStyle(el).visibility !== 'hidden';
  const main = document.querySelector('main.content');
  const label = el => {
    const by = el.getAttribute('aria-labelledby');
    return (el.getAttribute('aria-label') || (by && document.getElementById(by)?.textContent) ||
      (el.labels && [...el.labels].map(l => l.textContent).join(' ')) || el.getAttribute('title') ||
      el.getAttribute('placeholder') || el.textContent || '').trim();
  };
  const describe = el => {
    const r = el.getBoundingClientRect();
    const cls = typeof el.className === 'string' ? el.className : '';
    return `${el.tagName.toLowerCase()}.${cls.trim().replace(/\s+/g, '.')} "${label(el).slice(0, 30)}" ` +
      `${Math.round(r.width)}x${Math.round(r.height)}@${Math.round(r.left)}`;
  };
  // Links inside running text are exempt from the target size (WCAG 2.5.8).
  const inline = el => el.tagName === 'A' && getComputedStyle(el).display === 'inline';
  const controls = [...main.querySelectorAll(
    'button, a[href], input:not([type=hidden]), select, textarea, summary, [role=button], [role=radio], [role=checkbox], [role=tab], [tabindex]:not([tabindex="-1"])',
  )].filter(el => shown(el) && !el.disabled);
  const small = controls.filter(el => {
    if (inline(el)) return false;
    // A control inside a 48dp wrapper (a label around a checkbox) counts
    // as the wrapper.
    const hit = el.closest('label') || el;
    return hit.getBoundingClientRect().height < target - 0.5;
  });
  const knownAs = el => known.find(sel => el.matches(sel));
  const under48 = controls.filter(el => !inline(el) && (el.closest('label') || el).getBoundingClientRect().height < 47.5);
  const offscreen = controls.filter(el => {
    const r = el.getBoundingClientRect();
    // Inside a sideways scroller (a wide table) is reachable by scrolling.
    if (el.closest('.ls-table-scroll, .table-scroll, [data-scroll-x]')) return false;
    return r.left < -1 || r.right > innerWidth + 1;
  });
  return {
    direction: getComputedStyle(document.querySelector('.app')).direction,
    documentWidth: document.documentElement.scrollWidth,
    mainOverflow: main.scrollWidth - main.clientWidth,
    h1: [...main.querySelectorAll('h1')].filter(shown).map(h => h.textContent.trim()),
    back: [...main.querySelectorAll('[data-back="true"]')].filter(shown).map(describe),
    controls: controls.length,
    tooSmall: small.filter(el => !knownAs(el)).map(describe),
    target,
    under48: under48.length,
    knownSmall: [...new Set(small.map(knownAs).filter(Boolean))],
    offscreen: offscreen.map(describe),
    unnamed: controls.filter(el => !label(el)).map(describe),
  };
}

// What each screen state must show, beyond the shared layout rules.
async function screenChecks(page, item, name) {
  if (item.screen === 'learn') {
    const s = await page.evaluate(() => {
      const shown = [...document.querySelectorAll('.ls-stage')].filter(el => !el.hidden && el.getClientRects().length);
      const meter = document.querySelector('.ls-segs');
      const next = document.querySelector('.ls-nav [data-nav="next"], .ls-nav [data-nav="check"]');
      return {
        view: document.querySelector('.ls-player')?.dataset.view,
        stages: shown.map(el => [el.dataset.kind, el.dataset.n]),
        meter: meter && [meter.getAttribute('role'), meter.getAttribute('aria-valuenow'), meter.getAttribute('aria-valuemax')],
        next: Boolean(next && next.getClientRects().length),
        skip: Boolean(document.querySelector('.ls-nav .ls-skip')),
        hint: document.querySelector('#ls-gate-hint')?.className,
      };
    });
    assert.equal(s.view, item.view, `${name}: view`);
    assert.deepEqual(s.stages, [[item.kind, String(item.step)]], `${name}: exactly the saved step is shown`);
    assert.deepEqual(s.meter, ['progressbar', String(item.step), String(item.steps)], `${name}: step meter`);
    assert.ok(s.next, `${name}: a visible way forward`);
    if (item.question && item.kind !== 'case') {
      assert.ok(s.skip, `${name}: an unanswered question offers Skip`);
      assert.equal(s.hint, 'ls-sr-only', `${name}: the gate note is for screen readers only`);
    }
  } else {
    const s = await page.evaluate(() => ({
      kind: document.querySelector('.st-item')?.dataset.kind,
      bar: document.querySelector('.st-bar')?.getAttribute('role'),
      grade: document.querySelectorAll('.st-item button').length,
    }));
    if (item.phase === 'running') {
      assert.equal(s.kind, item.kind, `${name}: item kind`);
      assert.equal(s.bar, 'progressbar', `${name}: session progress`);
      assert.ok(s.grade > 0, `${name}: the item has a control`);
    }
  }
}

// Every lesson figure (learn/browser.rs `export_screen_fixtures_figures`):
// each label inside its picture. A label past the viewBox edge is cut off on
// the phone ("first decompensation: insidiou", 2026-11-01). The viewBox is
// in user units, so one width shows every overflow.
async function figureChecks(page, fixtures) {
  await page.setViewportSize({ width: 384, height: 900 });
  await page.goto(pathToFileURL(path.join(fixtures, 'figures', 'figures.html')).href);
  await page.evaluate(() => document.fonts.ready);
  const r = await page.evaluate(() => {
    const out = [];
    const figures = [...document.querySelectorAll('.ls-fig-svg')];
    for (const fig of figures) {
      const svg = fig.querySelector('svg');
      const vb = svg.viewBox.baseVal;
      const m = svg.getScreenCTM();
      const a = new DOMPoint(vb.x, vb.y).matrixTransform(m);
      const b = new DOMPoint(vb.x + vb.width, vb.y + vb.height).matrixTransform(m);
      const [left, right, top, bottom] = [Math.min(a.x, b.x), Math.max(a.x, b.x), Math.min(a.y, b.y), Math.max(a.y, b.y)];
      for (const t of svg.querySelectorAll('text')) {
        const box = t.getBoundingClientRect();
        if (!box.width) continue;
        const px = Math.max(left - box.left, box.right - right, top - box.top, box.bottom - bottom);
        if (px > 1) out.push(`${fig.dataset.at}: "${t.textContent.trim().slice(0, 50)}" runs ${Math.round(px)}px past the edge`);
      }
    }
    return { figures: figures.length, out };
  });
  assert.deepEqual(r.out, [], 'figure labels cut off at the picture edge');
  console.log(`Checked the labels of ${r.figures} lesson figures.`);
}

// `--figures-only`: just the figure-label check, for lesson pull requests
// (lesson-figures.yml). It needs only the figures fixture.
async function figuresOnly(fixtures) {
  const browser = await chromium.launch({ headless: true });
  try {
    const page = await browser.newPage({ reducedMotion: 'reduce' });
    await figureChecks(page, fixtures);
  } finally {
    await browser.close();
  }
}

async function run() {
  if (process.argv[2] === '--figures-only') return figuresOnly(path.resolve(process.argv[3]));
  const fixtures = path.resolve(process.argv[2]);
  const output = path.resolve(process.argv[3] || path.join(fixtures, 'results'));
  await fs.mkdir(output, { recursive: true });
  const browser = await chromium.launch({ headless: true });
  const page = await browser.newPage({ reducedMotion: 'reduce' });
  const errors = [];
  page.on('pageerror', error => errors.push(error.message));
  const results = [];
  const known = [];
  const seen = new Set();
  const desktopCaptured = new Set();
  let current = '';
  try {
    for (const screen of ['learn', 'study']) {
      const dir = path.join(fixtures, screen);
      const manifest = JSON.parse(await fs.readFile(path.join(dir, 'manifest.json'), 'utf8'));
      const knownSmall = KNOWN.small.filter(([s]) => s === screen).map(([, sel]) => sel);
      for (const item of manifest) {
        await page.goto(pathToFileURL(path.join(dir, item.file)).href);
        await page.evaluate(() => document.fonts.ready);
        for (const scheme of SCHEMES) {
          await page.emulateMedia({ colorScheme: scheme });
          for (const appearance of APPEARANCES) {
            for (const size of SIZES) {
              // The same switch the app makes: the prefs stylesheet and the
              // appearance key on .app (main.rs).
              await page.evaluate(([appearance, size]) => new Promise(resolve => {
                document.querySelector('.app').dataset.appearance = appearance;
                const link = document.querySelector('#prefs');
                const href = `prefs-system-${size}.css`;
                if (link.getAttribute('href') === href) return resolve();
                link.onload = resolve;
                link.setAttribute('href', href);
              }), [appearance, size]);
              for (const width of WIDTHS) {
                const name = `${item.file.replace(/\.html$/, '')}-${width}-${scheme}-${appearance}-${size}`;
                current = name;
                await page.setViewportSize({ width, height: 900 });
                const m = await page.evaluate(measure, [FLOOR[screen], knownSmall]);
                assert.equal(m.direction, 'rtl', name);
                const overflowKnown = KNOWN.overflow.find(k => k.file === item.file && k.width === width && k.size === size);
                const wide = m.documentWidth > width + 1 || m.mainOverflow > 1;
                if (overflowKnown && wide) {
                  seen.add(overflowKnown);
                  known.push({ name, gap: 'overflow', px: Math.max(m.documentWidth - width, m.mainOverflow) });
                } else {
                  assert.ok(m.documentWidth <= width + 1, `${name}: page wider than the screen ${JSON.stringify(m)}`);
                  assert.ok(m.mainOverflow <= 1, `${name}: content overflows sideways by ${m.mainOverflow}px`);
                  assert.deepEqual(m.offscreen, [], `${name}: controls outside the screen`);
                }
                const headingKnown = KNOWN.noHeading.find(k => k.screen === item.screen && k.phase === item.phase);
                if (headingKnown && !m.h1.length) {
                  seen.add(headingKnown);
                  known.push({ name, gap: 'no heading' });
                } else {
                  assert.ok(m.h1.length >= 1, `${name}: a visible heading`);
                }
                for (const sel of m.knownSmall) {
                  seen.add(KNOWN.small.find(([s, k]) => s === screen && k === sel));
                  known.push({ name, gap: `under ${FLOOR[screen]}`, selector: sel });
                }
                assert.ok(m.back.length >= 1 || item.phase === 'setup' || item.phase === 'summary',
                  `${name}: a visible Back target for Escape and Android Back`);
                assert.deepEqual(m.unnamed, [], `${name}: controls without a name`);
                assert.deepEqual(m.tooSmall, [], `${name}: controls under ${FLOOR[screen]} (${m.target}px)`);
                await screenChecks(page, item, name);
                // Running Study on a compact phone has a readable meter
                // above 48dp Undo/End controls, never an overlapped row.
                if (screen === 'study' && item.phase === 'running' && (width === 320 || width === 384)) {
                  const compact = await page.evaluate(() => {
                    const get = selector => {
                      const node = document.querySelector(selector);
                      if (!node || !node.getClientRects().length) return null;
                      const b = node.getBoundingClientRect();
                      return { left: b.left, right: b.right, top: b.top, bottom: b.bottom,
                        width: b.width, height: b.height };
                    };
                    return {
                      meter: get('.st-progress > .st-bar'),
                      count: get('.st-progress > .st-count'),
                      undo: get('.st-progress > .st-undo'),
                      end: get('.st-progress > button[data-back="true"]'),
                    };
                  });
                  const { meter, count, undo, end } = compact;
                  assert.ok(meter && count && end, `${name}: progress, count and exit stay accessible`);
                  assert.ok(meter.width >= 60 && meter.right <= width + 1 && meter.left >= -1,
                    `${name}: meter must retain a readable width: ${JSON.stringify(compact)}`);
                  assert.ok(meter.bottom <= end.top + 1 && count.bottom <= end.top + 1,
                    `${name}: action row must be below progress: ${JSON.stringify(compact)}`);
                  assert.ok(end.height >= 47.5 && end.left >= -1 && end.right <= width + 1,
                    `${name}: end stays inside the phone viewport`);
                  if (undo) {
                    assert.ok(undo.height >= 47.5 && (undo.right <= end.left + 1 || end.right <= undo.left + 1),
                      `${name}: Undo and End must not overlap`);
                  }
                }
                // Check a reduced Android WebView viewport after the IME
                // appears. The item must remain scrollable, and visible
                // grading controls must not be hidden below the gesture area.
                if (screen === 'study' && item.phase === 'running' && width === 384 &&
                    scheme === 'light' && appearance === 'calm' && size === 'normal') {
                  await page.setViewportSize({ width: 384, height: 520 });
                  const compactViewport = await page.evaluate(() => {
                    const main = document.querySelector('main.content');
                    main.scrollTop = 0;
                    const end = main.querySelector('.st-progress button[data-back="true"]');
                    const exit = end?.getBoundingClientRect();
                    const canScroll = main.scrollHeight > main.clientHeight;
                    main.scrollTop = main.scrollHeight;
                    const grades = [...main.querySelectorAll('.st-grades button')]
                      .filter(el => el.getClientRects().length > 0 && !el.disabled)
                      .map(el => {
                        const r = el.getBoundingClientRect();
                        return { top: r.top, bottom: r.bottom, left: r.left, right: r.right,
                          width: r.width, height: r.height };
                      });
                    return { end: exit && { top: exit.top, bottom: exit.bottom },
                      canScroll, viewportHeight: window.innerHeight, grades,
                      scrollWidth: document.documentElement.scrollWidth };
                  });
                  assert.ok(compactViewport.end && compactViewport.end.top >= -1 &&
                    compactViewport.end.bottom <= 521,
                    `${name}: End must remain reachable at top of short viewport: ${JSON.stringify(compactViewport)}`);
                  assert.ok(compactViewport.viewportHeight === 520 && compactViewport.scrollWidth <= 385,
                    `${name}: no horizontal overflow on a short phone: ${JSON.stringify(compactViewport)}`);
                  assert.deepEqual(compactViewport.grades.filter(g =>
                    g.height < 47.5 || g.left < -1 || g.right > 385 ||
                    (g.bottom > 521 && !compactViewport.canScroll)), [],
                    `${name}: visible grade targets must remain usable: ${JSON.stringify(compactViewport)}`);
                  await page.setViewportSize({ width: 384, height: 900 });
                }
                // The compact phone header must offer the same quick-search
                // action at a thumb-sized target, without overflowing 320dp.
                if (screen === 'study' && item.phase === 'setup' && (width === 320 || width === 384)) {
                  const action = await page.evaluate(() => {
                    const b = document.querySelector('.topbar-search');
                    if (!b) return null;
                    const r = b.getBoundingClientRect();
                    return { w: r.width, h: r.height, left: r.left, right: r.right, name: b.getAttribute('aria-label') };
                  });
                  assert.ok(action && action.w >= 47.5 && action.h >= 47.5 &&
                    action.left >= -1 && action.right <= width + 1 && action.name,
                    `${name}: mobile search must remain visible, named and at least 48dp: ${JSON.stringify(action)}`);
                }
                // The regular control audit scopes main.content, not the
                // navigation shell. Cover the bottom bar at phone widths.
                if (screen === 'study' && item.phase === 'setup' && (width === 320 || width === 384)) {
                  const chrome = await page.evaluate(() => {
                    const nav = document.querySelector('.app > .nav');
                    return [...nav.querySelectorAll('button.nav-item')].filter(el =>
                      el.getClientRects().length > 0 && getComputedStyle(el).display !== 'none')
                      .map(el => {
                        const r = el.getBoundingClientRect();
                        return { name: el.getAttribute('aria-label') || el.textContent.trim(),
                          width: r.width, height: r.height, left: r.left, right: r.right };
                      });
                  });
                  assert.ok(chrome.length >= 4, `${name}: bottom navigation missing: ${JSON.stringify(chrome)}`);
                  assert.deepEqual(chrome.filter(b => !b.name || b.width < 47.5 || b.height < 47.5 ||
                    b.left < -1 || b.right > width + 1), [],
                    `${name}: bottom tab unnamed, undersized, or off-screen`);
                }
                // More's real CSS is rendered from production styles,
                // even though this static fixture cannot open its UI.
                if (screen === 'study' && item.phase === 'setup' && width === 384 &&
                    scheme === 'light' && appearance === 'calm' && size === 'normal') {
                  const moreClose = await page.evaluate(() => {
                    const sheet = document.createElement('div');
                    sheet.className = 'more-sheet';
                    sheet.innerHTML = '<button class="btn btn-ghost more-close">إغلاق</button>';
                    document.querySelector('.app').append(sheet);
                    const r = sheet.querySelector('button').getBoundingClientRect();
                    const b = sheet.getBoundingClientRect();
                    sheet.remove();
                    return { width: r.width, height: r.height, sheetRight: b.right, sheetLeft: b.left };
                  });
                  assert.ok(moreClose.width > 0 && moreClose.height >= 47.5 &&
                    moreClose.sheetLeft >= -1 && moreClose.sheetRight <= width + 1,
                    `${name}: More Close must be 48dp and remain within screen: ${JSON.stringify(moreClose)}`);
                }
                // Simulate a soft-keyboard-reduced viewport for the real
                // palette CSS. Its results remain scrollable below the field.
                if (screen === 'study' && item.phase === 'setup' && width === 384 &&
                    scheme === 'light' && appearance === 'calm' && size === 'normal') {
                  await page.setViewportSize({ width: 384, height: 520 });
                  const popup = await page.evaluate(() => {
                    const dialog = document.createElement('section');
                    dialog.className = 'command-palette';
                    dialog.setAttribute('role', 'dialog');
                    dialog.innerHTML = '<header class="command-palette-head"><h2>Search</h2></header>' +
                      '<div class="command-search-row"><input class="command-search palette-search" aria-label="Search"></div>' +
                      '<nav class="command-results">' +
                      '<button class="palette-result">Result</button>'.repeat(12) + '</nav>';
                    document.body.append(dialog);
                    const r = dialog.getBoundingClientRect();
                    const input = dialog.querySelector('input').getBoundingClientRect();
                    const result = dialog.querySelector('.command-results');
                    const data = { top: r.top, bottom: r.bottom, inputBottom: input.bottom,
                      scrollable: getComputedStyle(result).overflowY === 'auto' };
                    dialog.remove();
                    return data;
                  });
                  assert.ok(popup.top >= 0 && popup.bottom <= 520 + 1 &&
                    popup.inputBottom < popup.bottom && popup.scrollable,
                    `${name}: palette must fit a phone viewport with on-screen keyboard: ${JSON.stringify(popup)}`);
                  await page.setViewportSize({ width: 384, height: 900 });
                }
                if (width === 384 && appearance === 'calm' && size === 'larger') {
                  await page.screenshot({ path: path.join(output, `${name}.png`), fullPage: true });
                }
                // The native Mac starts at 1280x820. From 840px up the lesson is
                // one centred reading column (learn.css, docs/LESSONS.md §
                // Reading layout): ~60-75 characters a line, the lesson name
                // above the card, Back/Next right under it.
                if (screen === 'learn' && (width === 1280 || width === 840) && scheme === 'light' &&
                    appearance === 'calm' && size === 'normal' && item.view === 'steps') {
                  const col = await page.evaluate(readingColumn);
                  assert.ok(col.stageWidth >= 480 && col.stageWidth <= 760 &&
                    col.contextBottom <= col.stageTop + 1 && (col.navGap >= 0 || col.navStuck) && col.navGap <= 48 &&
                    Math.abs(col.offCentre) <= 2,
                    `${name}: lesson must be one centred reading column with its controls under the card: ${JSON.stringify(col)}`);
                }
                // Real production SSR + styles, now including the desktop sidebar.
                // One representative Steps lesson and one running study item
                // give Mohamed Mac-width screenshots alongside the test report.
                if (width === 1600 && scheme === 'light' && appearance === 'calm' && size === 'normal' &&
                    !desktopCaptured.has(screen) &&
                    (screen === 'learn' ? item.view === 'steps' : item.phase === 'running')) {
                  const desktop = await page.evaluate(() => {
                    const nav = document.querySelector('.nav').getBoundingClientRect();
                    const main = document.querySelector('main.content');
                    const style = getComputedStyle(main);
                    const context = document.querySelector('.ls-player[data-view="steps"] > .ls-context');
                    const stage = document.querySelector('.ls-stage:not([hidden])');
                    const study = document.querySelector('.st:has(> .st-item)');
                    return {
                      navWidth: nav.width,
                      padStart: parseFloat(style.paddingInlineStart),
                      padEnd: parseFloat(style.paddingInlineEnd),
                      context: context && { left: context.getBoundingClientRect().left, width: context.getBoundingClientRect().width },
                      stage: stage && { right: stage.getBoundingClientRect().right, width: stage.getBoundingClientRect().width },
                      studyWidth: study?.getBoundingClientRect().width,
                    };
                  });
                  assert.ok(desktop.navWidth >= 200, `${name}: Mac labelled sidebar must be present`);
                  assert.ok(Math.abs(desktop.padStart - desktop.padEnd) < 1,
                    `${name}: Mac canvas must use both sides of the window: ${JSON.stringify(desktop)}`);
                  if (screen === 'learn') {
                    const col = await page.evaluate(readingColumn);
                    assert.ok(col.stageWidth >= 560 && col.stageWidth <= 760 &&
                      col.contextBottom <= col.stageTop + 1 && (col.navGap >= 0 || col.navStuck) && col.navGap <= 48 &&
                      Math.abs(col.offCentre) <= 2,
                    `${name}: a wide Mac window keeps one centred reading column, not a pane per side: ${JSON.stringify(col)}`);
                  } else {
                    assert.ok(desktop.studyWidth >= 1000,
                      `${name}: study card should use the wide desktop canvas: ${JSON.stringify(desktop)}`);
                  }
                  // A wide desktop can hide its rail without losing the
                  // running lesson/card, and can restore it without navigation.
                  const focus = await page.evaluate(() => {
                    const app = document.querySelector('.app');
                    const main = document.querySelector('main.content');
                    const nav = document.querySelector('nav.nav');
                    const toggle = main.querySelector('.desk-focus-toggle');
                    const before = main.getBoundingClientRect().width;
                    app.dataset.deskFocus = 'true';
                    const after = main.getBoundingClientRect().width;
                    const hidden = getComputedStyle(nav).display === 'none';
                    app.dataset.deskFocus = 'false';
                    return { before, after, hidden,
                      toggleVisible: Boolean(toggle && toggle.getClientRects().length) };
                  });
                  assert.ok(focus.hidden && focus.toggleVisible && focus.after > focus.before + 140,
                    `${name}: focus must widen the reading canvas and keep the toggle: ${JSON.stringify(focus)}`);

                  if (screen === 'study') {
                    const divider = page.locator('.desk-split .desk-resize').first();
                    assert.equal(await divider.count(), 1, `${name}: study has one pane separator`);
                    const before = await page.evaluate(() => {
                      const h = document.querySelector('.desk-resize');
                      return { visible: getComputedStyle(h).display !== 'none',
                        value: Number(h.getAttribute('aria-valuenow')) };
                    });
                    assert.ok(before.visible && before.value >= 35 && before.value <= 70,
                      `${name}: keyboard-adjustable separator visible on Mac`);
                    // Run the exact production pointer listener with a stub
                    // for the Rust bridge; SSR cannot save real preferences.
                    await page.evaluate(() => {
                      window.__paneMessages = [];
                      window.dioxus = { send: message => window.__paneMessages.push(message) };
                    });
                    const code = await fs.readFile(path.join(dir, 'desk-resize.js'), 'utf8');
                    await page.addScriptTag({ content: `(async () => {\n${code}\n})();` });
                    await divider.focus();
                    await page.keyboard.press('ArrowLeft'); // RTL: left widens the primary pane.
                    const keyed = await page.evaluate(() => Number(document.querySelector('.desk-resize').getAttribute('aria-valuenow')));
                    assert.equal(keyed, before.value + 5, `${name}: arrows adjust RTL primary width`);
                    await divider.dblclick(); // Reset without losing the active item.
                    const reset = await page.evaluate(() => ({
                      value: Number(document.querySelector('.desk-resize').getAttribute('aria-valuenow')),
                      custom: document.documentElement.style.getPropertyValue('--desk-split-width'),
                      sent: window.__paneMessages,
                    }));
                    assert.equal(reset.custom, '', `${name}: double click resets the split`);
                    assert.equal(reset.value, before.value, `${name}: reset restores the item default`);
                    assert.ok(reset.sent.includes('split:0') && reset.sent.includes('split:' + keyed),
                      `${name}: valid widths go through the preference bridge`);
                  }
                  await page.screenshot({ path: path.join(output, `mac-preview-${screen}-1600.png`) });
                  desktopCaptured.add(screen);
                }
                results.push({ ...item, width, scheme, appearance, size, controls: m.controls, under48: m.under48 });
              }
            }
          }
        }
      }
    }
    // Each group after the layout pass runs even when one before it fails
    // (#980: a clipped figure label hid the touch, focus and highlight
    // results for two scheduled runs). The failures are reported together.
    const failed = [];
    const group = async (name, check) => {
      current = name;
      try {
        await check();
      } catch (error) {
        console.error(`Failed at ${name}:\n${error.message}`);
        failed.push(name);
      }
    };
    await group('figures', () => figureChecks(page, fixtures));
    await group('compact touch controls', () => mobileControlChecks(page, fixtures));
    await group('keyboard focus', () => keyboardFocusChecks(browser));
    await group('own highlights', () => highlightChecks(browser, fixtures));
    await group('browser page errors', async () => assert.deepEqual(errors, [], 'browser page errors'));
    if (failed.length) {
      current = failed.join(', ');
      throw new Error(`${failed.length} check group(s) failed: ${current}`);
    }
    const stale = [...KNOWN.small, ...KNOWN.overflow, ...KNOWN.noHeading].filter(k => !seen.has(k));
    await fs.writeFile(path.join(output, 'report.json'),
      JSON.stringify({ cases: results.length, known, stale, results }, null, 2));
    const gaps = [...new Set(known.map(k => `${k.gap}${k.selector ? ` ${k.selector}` : ''}`))];
    console.log(`Passed ${results.length} production lesson player and study layout cases.`);
    const study48 = results.filter(r => r.screen === 'study' && r.under48 > 0).length;
    console.log(`Study cases with a control under 48px (sized by --tap): ${study48} of ${results.filter(r => r.screen === 'study').length}.`);
    console.log(`Known gaps still present (${gaps.length}):\n  ${gaps.join('\n  ')}`);
    for (const k of stale) console.warn(`::warning::Known gap no longer seen, remove it from KNOWN: ${JSON.stringify(k)}`);
  } catch (error) {
    console.error(`Failed at ${current}`);
    await page.screenshot({ path: path.join(output, 'failure.png'), fullPage: true }).catch(() => {});
    throw error;
  } finally {
    await browser.close();
  }
}

run().catch(error => { console.error(error); process.exitCode = 1; });
