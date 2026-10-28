// Batna hover peeks (PARITY.md §6.3, §6.4): a small card that shows while
// the S Pen hovers (in range, not touching) or the mouse rests:
//   - over an element with data-peek: its text (a book page's quote, an MCQ
//     option's `why` after answering);
//   - over any English word that is in the glossary: its expansion.
// Settings turn pen and mouse peeks off per device: data-pen-hover="off" /
// data-mouse-hover="off" on <html>. Touch never peeks.
// Adapted from kabar-app main: crates/app/assets/reader.js (pointer plumbing).
(() => {
  if (window.__batnaHover) return;
  const state = { exact: {}, lower: {}, unverified: "", timer: 0, shownFor: null };
  const card = document.createElement("div");
  card.className = "peek";
  card.setAttribute("role", "tooltip");
  card.hidden = true;
  document.body.appendChild(card);

  const root = document.documentElement;
  const enabled = (type) =>
    (type === "pen" ? root.dataset.penHover : root.dataset.mouseHover) !== "off";

  // The word under the pointer, from the text itself: the DOM is Dioxus's,
  // so nothing is wrapped or changed.
  function wordAt(x, y) {
    let node, offset;
    if (document.caretPositionFromPoint) {
      const p = document.caretPositionFromPoint(x, y);
      if (!p) return null;
      node = p.offsetNode;
      offset = p.offset;
    } else if (document.caretRangeFromPoint) {
      const r = document.caretRangeFromPoint(x, y);
      if (!r) return null;
      node = r.startContainer;
      offset = r.startOffset;
    } else return null;
    if (!node || node.nodeType !== Node.TEXT_NODE) return null;
    const text = node.textContent;
    const isWord = (c) => /[A-Za-z0-9-]/.test(c);
    let a = offset, b = offset;
    while (a > 0 && isWord(text[a - 1])) a--;
    while (b < text.length && isWord(text[b])) b++;
    if (a === b) return null;
    // The caret lands beside the nearest letter: make sure the pointer is
    // really on the word, not in the margin next to it.
    const range = document.createRange();
    range.setStart(node, a);
    range.setEnd(node, b);
    const rect = range.getBoundingClientRect();
    if (x < rect.left - 2 || x > rect.right + 2 || y < rect.top - 2 || y > rect.bottom + 2) return null;
    return { word: text.slice(a, b).replace(/^-+|-+$/g, ""), rect, node, a };
  }

  function lookup(word) {
    if (!word) return null;
    return state.exact[word] || state.lower[word.toLowerCase()] || null;
  }

  function show(text, note, rect) {
    card.textContent = "";
    const body = document.createElement("span");
    body.lang = "en";
    body.dir = "ltr";
    body.textContent = text;
    card.appendChild(body);
    if (note) {
      const n = document.createElement("small");
      n.className = "peek-note";
      n.textContent = note;
      card.appendChild(n);
    }
    card.hidden = false;
    const w = card.offsetWidth, h = card.offsetHeight;
    const vw = window.innerWidth, vh = window.innerHeight;
    let left = Math.min(Math.max(8, rect.left + rect.width / 2 - w / 2), vw - w - 8);
    let top = rect.top - h - 8;
    if (top < 8) top = Math.min(rect.bottom + 8, vh - h - 8);
    card.style.left = left + "px";
    card.style.top = top + "px";
  }

  function hide() {
    clearTimeout(state.timer);
    card.hidden = true;
    state.shownFor = null;
  }

  function peekAt(x, y, target) {
    const el = target && target.closest ? target.closest("[data-peek]") : null;
    if (el && el.dataset.peek) {
      if (state.shownFor === el) return;
      state.shownFor = el;
      show(el.dataset.peek, el.dataset.peekNote || "", el.getBoundingClientRect());
      return;
    }
    const hit = wordAt(x, y);
    const entry = hit && lookup(hit.word);
    if (!entry) return hide();
    const key = hit.word + "@" + hit.a;
    if (state.shownFor === key) return;
    state.shownFor = key;
    show(entry[0], entry[1] ? "" : state.unverified, hit.rect);
  }

  document.addEventListener("pointermove", (e) => {
    // Hover is the mouse, or a pen in range that is not touching the screen.
    const hovering = e.pointerType === "mouse" || (e.pointerType === "pen" && e.buttons === 0);
    if (!hovering) return;
    if (!enabled(e.pointerType)) return hide();
    clearTimeout(state.timer);
    const { clientX: x, clientY: y, target } = e;
    state.timer = setTimeout(() => peekAt(x, y, target), e.pointerType === "pen" ? 60 : 250);
  }, { passive: true });
  document.addEventListener("pointerdown", hide, { passive: true });
  document.addEventListener("pointerleave", hide, { passive: true });
  document.addEventListener("scroll", hide, { passive: true, capture: true });
  window.addEventListener("blur", hide);

  window.__batnaHover = {
    // exact: { term: [text, approved] } for abbreviations and drugs;
    // lower: { lowercased drug: [text, approved] }.
    setGlossary(g, unverifiedNote) {
      state.exact = g.exact || {};
      state.lower = g.lower || {};
      state.unverified = unverifiedNote || "";
    },
    hide,
    visible: () => !card.hidden,
  };
})();
