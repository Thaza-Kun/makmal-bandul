// Offline shell. Hashed WASM/JS/CSS assets are discovered from index.html at
// install time, so their build-dependent names never appear here.
const CACHE = 'makmal-bandul-v1';
const SHELL = [
  './',
  './index.html',
  './manifest.json',
  './ui-text.en.json',
  './ui-text.ms.json',
  './icon-192.png',
  './icon-512.png',
  './sw.js'
];

// The SW registers after the initial page has loaded, so its first fetch
// handler never sees the hashed JS/WASM. Read them from the built index.html
// and cache them now instead of waiting for a second visit.
async function precacheBundle(cache) {
  const index = await cache.match('./index.html');
  if (!index) return;
  const html = await index.text();
  const assets = [...html.matchAll(/(?:src|href)="([^"]+)"/g)]
    .map((match) => match[1])
    .filter((url) => /\.(js|wasm|css)$/.test(url))
    .map((url) => new URL(url, self.registration.scope).href);
  await Promise.all(assets.map((url) => cache.add(url).catch(() => {})));
}

self.addEventListener('install', (event) => {
  event.waitUntil(
    caches.open(CACHE)
      .then(async (cache) => {
        await cache.addAll(SHELL);
        await precacheBundle(cache);
      })
      .then(() => self.skipWaiting())
  );
});

self.addEventListener('activate', (event) => {
  event.waitUntil(
    caches.keys()
      .then((keys) => Promise.all(keys.filter((key) => key !== CACHE).map((key) => caches.delete(key))))
      .then(() => self.clients.claim())
  );
});

self.addEventListener('fetch', (event) => {
  const request = event.request;
  if (request.method !== 'GET') return;
  const url = new URL(request.url);
  if (url.origin !== self.location.origin) return;

  event.respondWith(
    caches.match(request).then((cached) => cached || fetch(request).then((response) => {
      if (response.ok) {
        const copy = response.clone();
        caches.open(CACHE).then((cache) => cache.put(request, copy));
      }
      return response;
    }).catch(() => caches.match('./index.html')))
  );
});
