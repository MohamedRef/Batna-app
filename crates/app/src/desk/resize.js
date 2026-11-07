// Pointer + keyboard resizing for the desktop study panes. The Rust app
// keeps the saved percentage in device-prefs.txt; this script only owns drag
// feedback and sends one value when the gesture finishes.
if (window.__batnaDeskResize) {
  for (const [name, fn] of Object.entries(window.__batnaDeskResize)) {
    document.removeEventListener(name, fn, true);
  }
}
const root = document.documentElement;
const fineDesktop = () => matchMedia("(min-width: 1200px) and (hover: hover) and (pointer: fine)").matches;
const bound = n => Math.min(70, Math.max(35, Math.round(n)));
let drag = null;
const apply = (value, save) => {
  const pct = bound(value);
  root.style.setProperty("--desk-split-width", pct + "%");
  document.querySelectorAll(".desk-resize").forEach(h => {
    h.setAttribute("aria-valuenow", String(pct));
    h.setAttribute("aria-valuetext", pct + "%");
  });
  if (save) dioxus.send("split:" + pct);
};
const pointerdown = e => {
  const handle = e.target.closest?.(".desk-resize");
  if (!handle || !fineDesktop() || e.button !== 0) return;
  const split = handle.closest(".desk-split");
  if (!split) return;
  drag = { handle, split, pointerId: e.pointerId };
  handle.setPointerCapture?.(e.pointerId);
  root.classList.add("desk-resizing");
  e.preventDefault();
};
const pointermove = e => {
  if (!drag || e.pointerId !== drag.pointerId) return;
  const b = drag.split.getBoundingClientRect();
  if (!b.width) return;
  const rtl = getComputedStyle(drag.split).direction === "rtl";
  const x = rtl ? b.right - e.clientX : e.clientX - b.left;
  apply(100 * x / b.width, false);
};
const pointerup = e => {
  if (!drag || e.pointerId !== drag.pointerId) return;
  const handle = drag.handle;
  drag = null;
  root.classList.remove("desk-resizing");
  try { if (handle.hasPointerCapture?.(e.pointerId)) handle.releasePointerCapture(e.pointerId); } catch (_) {}
  const value = Number(root.style.getPropertyValue("--desk-split-width").replace("%", ""));
  if (Number.isFinite(value)) apply(value, true);
};
const pointercancel = e => {
  if (!drag || e.pointerId !== drag.pointerId) return;
  drag = null;
  root.classList.remove("desk-resizing");
  // Keep the last valid width, but do not save an interrupted drag.
};
const keydown = e => {
  const handle = e.target.closest?.(".desk-resize");
  if (!handle || !fineDesktop() || e.altKey || e.ctrlKey || e.metaKey || e.shiftKey) return;
  const rtl = getComputedStyle(handle).direction === "rtl";
  const diff = e.key === "ArrowLeft" ? (rtl ? 5 : -5) :
    e.key === "ArrowRight" ? (rtl ? -5 : 5) : 0;
  if (!diff && e.key !== "Home" && e.key !== "End") return;
  e.preventDefault();
  const current = Number(handle.getAttribute("aria-valuenow")) || Number(handle.dataset.default) || 60;
  apply(e.key === "Home" ? 35 : e.key === "End" ? 70 : current + diff, true);
};
const dblclick = e => {
  const handle = e.target.closest?.(".desk-resize");
  if (!handle || !fineDesktop()) return;
  root.style.removeProperty("--desk-split-width");
  document.querySelectorAll(".desk-resize").forEach(h => {
    const value = Number(h.dataset.default) || 60;
    h.setAttribute("aria-valuenow", String(value));
    h.setAttribute("aria-valuetext", value + "%");
  });
  dioxus.send("split:0");
};
const listeners = { pointerdown, pointermove, pointerup, pointercancel, keydown, dblclick };
window.__batnaDeskResize = listeners;
for (const [name, fn] of Object.entries(listeners)) document.addEventListener(name, fn, true);
await new Promise(() => {});