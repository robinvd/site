const html = document.querySelector('html');
const colorSelector = document.querySelector('#mode-switcher');
function switchMode(mode) {
  localStorage.setItem("mode", mode);
  html.style.setProperty("color-scheme", mode === "auto" ? "light dark" : mode);
  colorSelector.value = mode
}
switchMode(localStorage.getItem("mode") || "auto");
function prefetch(target) {
  const href = target.href;
  const link = document.createElement('link');
  for (const node of document.head.children) {
    if (node.href === href) {
      return
    }
  }
  link.rel = 'preload';
  link.href = href;
  document.head.appendChild(link);
}
