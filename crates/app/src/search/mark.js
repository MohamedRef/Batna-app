// Marks the searched words in a lesson step (#1350) with the CSS Custom
// Highlight API: ranges over the step's text, no change to its DOM, so the
// page's own rendering never sees it. Folding matches the search's:
// case, hamza forms, ة/ه, ى/ي, ؤ/و, tashkeel, Arabic digits and dashes.
window.__batnaFind = window.__batnaFind || function (words, n) {
  const fold = (c) => {
    if (/[ً-ٰٟـ]/.test(c)) return "";
    if ("أإآٱ".includes(c)) return "ا";
    if (c === "ة") return "ه";
    if (c === "ى" || c === "ئ") return "ي";
    if (c === "ؤ") return "و";
    if ("-‐‑‒–—−/".includes(c)) return " ";
    const d = "٠١٢٣٤٥٦٧٨٩".indexOf(c);
    if (d >= 0) return String(d);
    return c.toLowerCase();
  };
  let tries = 0;
  const run = () => {
    const step = document.querySelector('article.ls-stage[data-n="' + n + '"]');
    if (!step || step.hidden) {
      if (++tries < 40) setTimeout(run, 75);
      return;
    }
    if (!(window.CSS && CSS.highlights && window.Highlight)) return;
    const ranges = [];
    const walk = document.createTreeWalker(step, NodeFilter.SHOW_TEXT);
    while (walk.nextNode()) {
      const node = walk.currentNode;
      const text = node.data;
      let f = "";
      const at = [];
      for (let i = 0; i < text.length; i++) {
        const piece = fold(text[i]);
        for (let k = 0; k < piece.length; k++) at.push(i);
        f += piece;
      }
      for (const w of words) {
        let from = 0;
        for (;;) {
          const hit = f.indexOf(w, from);
          if (hit < 0 || !w) break;
          const r = document.createRange();
          r.setStart(node, at[hit]);
          r.setEnd(node, at[hit + w.length - 1] + 1);
          ranges.push(r);
          from = hit + w.length;
        }
      }
    }
    CSS.highlights.set("batna-find", new Highlight(...ranges));
    const first = ranges[0] && ranges[0].startContainer.parentElement;
    if (first) first.scrollIntoView({ block: "center", behavior: "smooth" });
  };
  run();
};
