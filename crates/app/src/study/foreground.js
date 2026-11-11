// A single app-lifetime bridge, not a listener per Study screen mount.
// On Android WebView visibility is typically signalled on Activity resume;
// pageshow also covers returning from the browser's page cache. Neither
// changes the current question, the scroll container or Android Back state.
window.__batnaStudyForegroundSend = (message) => dioxus.send(message);
if (!window.__batnaStudyForegroundInit) {
  window.__batnaStudyForegroundInit = true;
  const visible = () => {
    if (document.visibilityState === "visible") {
      window.__batnaStudyForegroundSend("visible");
    }
  };
  document.addEventListener("visibilitychange", visible);
  window.addEventListener("pageshow", visible);
}
// Keep the Dioxus eval channel open for the app's lifetime, like Back.
await new Promise(() => {});
