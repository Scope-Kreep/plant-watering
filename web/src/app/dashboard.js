function human(secs) {
  const s = Math.max(0, Math.floor(secs));
  if (s < 60) return s + "s";
  if (s < 3600) return Math.floor(s / 60) + "m";
  if (s < 86400)
    return Math.floor(s / 3600) + "h " + Math.floor((s % 3600) / 60) + "m";
  return Math.floor(s / 86400) + "d " + Math.floor((s % 86400) / 3600) + "h";
}

setInterval(() => {
  const now = Date.now() / 1000;
  document.querySelectorAll("[data-updated-at]").forEach((el) => {
    const s = Number(el.dataset.updatedAt);
    if (s) el.textContent = "updated " + human(now - s) + " ago";
  });
}, 1000);
