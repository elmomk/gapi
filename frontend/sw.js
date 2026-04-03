const CACHE_NAME = 'garmin-dashboard-v1';

self.addEventListener('install', (event) => {
  event.waitUntil(
    caches.open(CACHE_NAME).then((cache) => cache.addAll(['/']))
  );
  self.skipWaiting();
});

self.addEventListener('activate', (event) => {
  event.waitUntil(
    caches.keys().then((names) => Promise.all(
      names.filter((n) => n !== CACHE_NAME).map((n) => caches.delete(n))
    ))
  );
  self.clients.claim();
});

self.addEventListener('fetch', (event) => {
  const url = new URL(event.request.url);

  // Cache-first for hashed static assets (immutable)
  if (url.pathname.match(/\.(js|wasm|css)$/) && url.pathname.match(/-[a-f0-9]{16}/)) {
    event.respondWith(
      caches.match(event.request).then((cached) => {
        if (cached) return cached;
        return fetch(event.request).then((resp) => {
          caches.open(CACHE_NAME).then((c) => c.put(event.request, resp.clone()));
          return resp;
        });
      })
    );
    return;
  }

  // Don't cache config or API responses
  if (url.pathname === '/config.json' || url.pathname.startsWith('/api/')) {
    event.respondWith(fetch(event.request).catch(() => new Response('offline')));
    return;
  }

  // Network-first for everything else
  event.respondWith(
    fetch(event.request).then((resp) => {
      if (event.request.method === 'GET') {
        caches.open(CACHE_NAME).then((c) => c.put(event.request, resp.clone()));
      }
      return resp;
    }).catch(() => caches.match(event.request).then((c) => c || caches.match('/')))
  );
});
