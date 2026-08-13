/**
 * Prefetches images via the browser HTTP cache.
 * Uses `new Image()` instead of `<link rel="prefetch">` because Safari
 * does not support rel=prefetch.
 */

// Keep references so in-flight requests are not aborted by GC,
// and so the same URL is never fetched twice per session.
const prefetchedImages = new Map<string, HTMLImageElement>()

export function prefetchImage(url: string): void {
  if (prefetchedImages.has(url)) return
  const img = new Image()
  img.decoding = 'async'
  img.src = url
  prefetchedImages.set(url, img)
}
