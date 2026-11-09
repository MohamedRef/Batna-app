// Batna info sheet gestures (crates/app/src/sheet.rs). Adapted from
// kabar-app main: crates/app/assets/sheet.js (PR #453).
//   handle : drag freely; let go near a stop (peek / half / near full) and
//            it settles there; fling up or down to the next stop; drag or
//            fling below the peek to close. Double-tap toggles half ↔ full.
//            Arrow keys step between stops, Home/End jump to the ends,
//            Enter toggles half ↔ full.
//   body   : scrolled to the top, pulling down drags the sheet too.
//   Esc    : closes the sheet before anything under it.
//   outside: a tap outside the sheet closes it (and does nothing else).
//   tablet / Mac (from 600dp wide and 480dp tall): the sheet is a floating
//            card (.float) placed by the chip that opened it; dragging its
//            heading row moves it, and it never grows past the room it has
//            (Mohamed, 2026-11-09: docked at the foot, a drag up made it a
//            very tall strip).
// The size is a fraction of the window height (--sheet-f), remembered for
// the next sheet. Listeners are delegated from document, installed once.
if (!window.__batnaSheetInit) {
  window.__batnaSheetInit = true;
  const STOPS = [0.32, 0.56, 0.9];
  const MIN = 0.12; // let go below this and the sheet closes
  const MAX = 0.94;
  const PULL = 0.05; // a stop this close catches the sheet
  const FLING = 0.5; // px per ms
  const KEY = "batna.sheet.f";

  // Until he picks a size, a sheet is as tall as its content (at most
  // half the window). Once he has dragged it, it keeps the height he chose,
  // even when its content is shorter, so a drag always does something.
  let last = 0.56;
  let chosen = false;
  try {
    const v = Number(localStorage.getItem(KEY));
    if (v >= STOPS[0] && v <= MAX) {
      last = v;
      chosen = true;
    }
  } catch (_) {}
  const remember = (f) => {
    last = f;
    chosen = true;
    try { localStorage.setItem(KEY, String(f)); } catch (_) {}
  };

  const vh = () => window.innerHeight || document.documentElement.clientHeight;
  const sheets = () => [...document.querySelectorAll(".bsheet")];
  const clamp = (f) => Math.min(MAX, Math.max(0.05, f));
  // The fraction the sheet really covers (short content stays short).
  const shown = (el) => el.offsetHeight / vh();
  const reduced = () => matchMedia("(prefers-reduced-motion: reduce)").matches;

  // ---- floating card (tablet and Mac) ----
  const FLOAT = matchMedia("(min-width: 600px) and (min-height: 480px)");
  const EDGE = 12; // px kept free at the window's edges
  const GAP = 8; // px between the chip and the card
  const vw = () => window.innerWidth || document.documentElement.clientWidth;
  const rem = () => parseFloat(getComputedStyle(document.documentElement).fontSize) || 16;
  const rtl = (el) => getComputedStyle(el).direction === "rtl";
  // What was tapped last, so the card opens next to it.
  let anchor = null; // { el, t }
  const noteTap = (e) => {
    const t = e.target instanceof Element ? e.target : null;
    if (!t || t.closest(".bsheet")) return;
    const el = t.closest("[aria-haspopup], button, a, [role=button]") || t;
    anchor = { el, t: performance.now() };
  };
  const anchorRect = () => {
    if (!anchor || !anchor.el.isConnected || performance.now() - anchor.t > 3000) return null;
    const r = anchor.el.getBoundingClientRect();
    if (!r.width || r.bottom < 0 || r.top > vh() || r.right < 0 || r.left > vw()) return null;
    return r;
  };
  // Places the card at `left` / `top` (window px), kept inside the window.
  // The CSS reads its place from the inline start, so RTL flips it.
  const setPos = (el, left, top) => {
    const w = el.offsetWidth;
    const h = el.offsetHeight;
    left = Math.min(Math.max(EDGE, left), Math.max(EDGE, vw() - w - EDGE));
    top = Math.min(Math.max(EDGE, top), Math.max(EDGE, vh() - h - EDGE));
    el.style.setProperty("--float-x", `${Math.round(rtl(el) ? vw() - left - w : left)}px`);
    el.style.setProperty("--float-y", `${Math.round(top)}px`);
  };
  // Below the chip if it fits, else above it, else on the side with more
  // room (its content scrolls), else in the middle of the window.
  const place = (el) => {
    if (!FLOAT.matches) {
      el.classList.remove("float", "placed");
      for (const v of ["--float-x", "--float-y", "--float-h", "--float-origin"]) el.style.removeProperty(v);
      return;
    }
    el.classList.add("float");
    const cap = Math.min(36 * rem(), 0.76 * vh());
    el.style.setProperty("--float-h", `${Math.round(cap)}px`);
    const w = el.offsetWidth;
    const h = el.offsetHeight;
    const r = anchorRect();
    let left = (vw() - w) / 2;
    let top = (vh() - h) / 2;
    let origin = "center";
    if (r) {
      left = rtl(el) ? r.right - w : r.left;
      const below = vh() - r.bottom - GAP - EDGE;
      const above = r.top - GAP - EDGE;
      const side = rtl(el) ? "right" : "left";
      if (h <= below || (below >= above && below >= 260)) {
        if (h > below) el.style.setProperty("--float-h", `${Math.round(below)}px`);
        top = r.bottom + GAP;
        origin = `top ${side}`;
      } else if (h <= above || above >= 260) {
        const room = Math.min(h, above);
        if (h > above) el.style.setProperty("--float-h", `${Math.round(above)}px`);
        top = r.top - GAP - room;
        origin = `bottom ${side}`;
      }
    }
    el.style.setProperty("--float-origin", origin);
    setPos(el, left, top);
    el.classList.add("placed");
  };
  // Keeps a placed card inside the window when its content or the window
  // changes size; crossing the tablet width switches sheet <-> card.
  const keepIn = (el) => {
    if (FLOAT.matches !== el.classList.contains("float")) return place(el);
    if (!el.classList.contains("placed")) return;
    const r = el.getBoundingClientRect();
    setPos(el, r.left, r.top);
  };

  // The sheet floats over the page (Mohamed, 2026-10-29: it overlays the
  // content, it never pushes it up), so it adds no room under the page and
  // scrolls nothing. `data-sheet` only says one is open.
  const room = () => {
    const open = sheets().some((el) => !el.classList.contains("closing"));
    if (open) document.documentElement.dataset.sheet = "open";
    else delete document.documentElement.dataset.sheet;
    sheets().forEach(keepIn);
  };

  const setF = (el, f, settle) => {
    el.classList.toggle("settling", !!settle && !reduced());
    el.style.setProperty("--sheet-f", clamp(f).toFixed(4));
    if (settle) setTimeout(room, 260);
    else room();
  };

  const close = (el) => {
    if (el.classList.contains("closing")) return;
    const btn = el.querySelector("[data-sheet-close]");
    if (reduced()) {
      btn?.click();
      return;
    }
    el.classList.add("closing");
    room();
    setTimeout(() => btn?.click(), 170);
  };

  // Settle after a drag: fling to the next stop, else stay where let go
  // unless a stop is near. Below the floor, close.
  const settle = (el, f, v) => {
    if (v > FLING) {
      const lower = [...STOPS].reverse().find((s) => s < f - 0.02);
      if (lower === undefined) return close(el);
      return land(el, lower);
    }
    if (v < -FLING) {
      const higher = STOPS.find((s) => s > f + 0.02);
      return land(el, higher ?? MAX);
    }
    if (f < MIN) return close(el);
    const near = STOPS.find((s) => Math.abs(s - f) <= PULL);
    land(el, near ?? f);
  };
  const land = (el, f) => {
    el.classList.add("sized");
    setF(el, f, true);
    remember(clamp(f));
  };

  // ---- dragging: pointer on the handle, touch on the body ----
  let drag = null; // { el, y0, f0, t, y, v, moved, kind }

  const start = (el, y) => {
    drag = { el, y0: y, f0: shown(el), y, t: performance.now(), v: 0, moved: false };
    if (el.classList.contains("float")) {
      const r = el.getBoundingClientRect();
      drag.left0 = r.left;
      drag.top0 = r.top;
      el.classList.add("dragging");
      return;
    }
    el.classList.add("dragging", "sized");
    el.classList.remove("settling");
  };
  const move = (y, x) => {
    if (drag.left0 !== undefined) {
      if (drag.x0 === undefined) drag.x0 = x;
      drag.y = y;
      if (Math.abs(y - drag.y0) + Math.abs(x - drag.x0) > 4) drag.moved = true;
      setPos(drag.el, drag.left0 + x - drag.x0, drag.top0 + y - drag.y0);
      return;
    }
    const now = performance.now();
    const dt = Math.max(1, now - drag.t);
    // Smoothed velocity, positive downwards.
    drag.v = 0.7 * ((y - drag.y) / dt) + 0.3 * drag.v;
    drag.y = y;
    drag.t = now;
    if (Math.abs(y - drag.y0) > 4) drag.moved = true;
    setF(drag.el, drag.f0 - (y - drag.y0) / vh(), false);
  };
  const end = () => {
    const d = drag;
    drag = null;
    d.el.classList.remove("dragging");
    if (d.left0 !== undefined) return { ...d, moved: true }; // a card: moved, never sized
    if (d.moved) settle(d.el, d.f0 - (d.y - d.y0) / vh(), d.v);
    return d;
  };

  let lastTap = 0;
  document.addEventListener("pointerdown", (e) => {
    const grip = e.target instanceof Element && e.target.closest(".bsheet-grip, .bsheet-head");
    if (!grip || e.button > 0 || e.target.closest("button")) return;
    const el = grip.closest(".bsheet");
    if (!el) return;
    e.preventDefault();
    grip.setPointerCapture?.(e.pointerId);
    start(el, e.clientY);
    if (drag.left0 !== undefined) drag.x0 = e.clientX;
  });
  document.addEventListener("pointermove", (e) => {
    if (drag && drag.kind !== "touch") move(e.clientY, e.clientX);
  });
  const up = () => {
    if (!drag || drag.kind === "touch") return;
    const d = end();
    if (d.moved) return;
    // A tap on the handle: a double tap toggles half and near full.
    const now = performance.now();
    if (now - lastTap < 350) {
      const f = shown(d.el);
      land(d.el, f < (STOPS[1] + STOPS[2]) / 2 ? STOPS[2] : STOPS[1]);
      lastTap = 0;
    } else {
      lastTap = now;
    }
  };
  document.addEventListener("pointerup", up);
  document.addEventListener("pointercancel", up);

  // Pull-down from the top of the body. Touch events, because the body
  // must keep scrolling natively until the pull starts.
  let pull = null; // { el, body, y0 }
  document.addEventListener(
    "touchstart",
    (e) => {
      const body = e.target instanceof Element && e.target.closest(".bsheet-body");
      const el = body?.closest(".bsheet");
      if (!el || e.touches.length > 1 || el.classList.contains("float")) return;
      pull = { el, body, y0: e.touches[0].clientY };
    },
    { passive: true },
  );
  document.addEventListener(
    "touchmove",
    (e) => {
      if (!pull) return;
      const y = e.touches[0].clientY;
      if (!drag) {
        if (pull.body.scrollTop > 0 || y - pull.y0 < 8) {
          if (pull.y0 - y > 8) pull = null; // scrolling up: leave it be
          return;
        }
        start(pull.el, y);
        drag.kind = "touch";
      }
      e.preventDefault();
      move(y);
    },
    { passive: false },
  );
  const touchEnd = () => {
    if (drag && drag.kind === "touch") end();
    pull = null;
  };
  document.addEventListener("touchend", touchEnd);
  document.addEventListener("touchcancel", touchEnd);

  // ---- keyboard (on window, before the app's shortcuts on document) ----
  window.addEventListener("keydown", (e) => {
    const grip = e.target instanceof Element && e.target.closest(".bsheet-grip");
    if (!grip) return;
    const el = grip.closest(".bsheet");
    if (!el) return;
    const f = shown(el);
    let to = null;
    if (e.key === "ArrowUp") to = STOPS.find((s) => s > f + 0.02) ?? MAX;
    else if (e.key === "ArrowDown") to = [...STOPS].reverse().find((s) => s < f - 0.02) ?? STOPS[0];
    else if (e.key === "Home") to = MAX;
    else if (e.key === "End") to = STOPS[0];
    else if (e.key === "Enter" || e.key === " ") to = f < (STOPS[1] + STOPS[2]) / 2 ? STOPS[2] : STOPS[1];
    if (to !== null) {
      e.preventDefault();
      e.stopPropagation();
      land(el, to);
    }
  }, true);
  // Esc closes the sheet first: the page under it is not modal, so the
  // app's shortcuts would otherwise act on it.
  window.addEventListener("keydown", (e) => {
    if (e.key !== "Escape" || e.altKey || e.ctrlKey || e.metaKey) return;
    if (document.querySelector("[role=dialog][aria-modal=true]")) return;
    const el = sheets().find((s) => !s.classList.contains("closing"));
    if (!el) return;
    e.preventDefault();
    e.stopPropagation();
    close(el);
  }, true);

  // ---- a tap outside closes it (Mohamed, 2026-11-03) ----
  // The page under the sheet still scrolls (a scroll is no click), but a
  // tap anywhere outside closes the sheet and does nothing else, so a tap
  // meant to dismiss never answers a question or leaves the screen. A tap
  // on something that opens a sheet or menu (a "ص N" chip, the
  // abbreviations chip, the course chip) still works: it opens its own
  // sheet in place of this one. Taps inside another dialog, menu, peek
  // card or backdrop are theirs. Keyboard and scripted clicks don't count.
  const PASS = "[aria-haspopup], [role=dialog], [role=menu], [role=tooltip], [class*=backdrop], [class*=scrim]";
  window.addEventListener("pointerdown", noteTap, true);
  window.addEventListener("click", (e) => {
    noteTap(e);
    // A chip tapped while a card is open fills that card with its own
    // content: move the card next to the new chip.
    const t = e.target instanceof Element ? e.target : null;
    if (t && !t.closest(".bsheet") && t.closest("[aria-haspopup]")) {
      setTimeout(() => sheets().filter((s) => s.classList.contains("placed")).forEach(place), 60);
    }
  }, true);
  window.addEventListener("click", (e) => {
    if (!e.isTrusted || e.detail === 0) return;
    const t = e.target instanceof Element ? e.target : null;
    if (!t || t.closest(".bsheet") || t.closest(PASS)) return;
    const open = sheets().filter((s) => !s.classList.contains("closing"));
    if (!open.length) return;
    e.preventDefault();
    e.stopPropagation();
    open.forEach(close);
  }, true);

  // ---- a sheet opens ----
  // One sheet at a time; it opens at the size last left, over the page.
  const opened = (el) => {
    for (const other of sheets()) {
      if (other !== el) other.querySelector("[data-sheet-close]")?.click();
    }
    el.classList.toggle("sized", chosen);
    // Hidden until placed, so a card never flashes at the window's foot.
    if (FLOAT.matches) el.classList.add("float");
    setF(el, last, false);
    requestAnimationFrame(() => {
      place(el);
      room();
    });
  };
  new MutationObserver((records) => {
    let changed = false;
    for (const r of records) {
      for (const n of r.addedNodes) {
        if (!(n instanceof Element)) continue;
        const el = n.classList.contains("bsheet") ? n : n.querySelector(".bsheet");
        if (el) opened(el);
      }
      if (r.removedNodes.length) changed = true;
    }
    if (changed) room();
  }).observe(document.body, { childList: true, subtree: true });
  // Content inside changes height (another page opened in the same sheet).
  if (window.ResizeObserver) {
    const ro = new ResizeObserver(() => room());
    new MutationObserver(() => sheets().forEach((el) => ro.observe(el))).observe(document.body, {
      childList: true,
      subtree: true,
    });
  }
  window.addEventListener("resize", room);
}
