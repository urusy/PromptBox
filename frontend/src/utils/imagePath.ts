/**
 * Validates and sanitizes image paths from API responses.
 * Prevents path traversal and ensures paths follow expected format.
 */

/**
 * Extracts the basename from a file path.
 * Handles both Windows (backslash) and Unix (forward slash) paths.
 * Also removes common file extensions like .safetensors, .ckpt, .pt
 */
export function getBasename(path: string | null | undefined): string {
  if (!path) return ''

  // Split by both backslash and forward slash, take the last part
  const parts = path.split(/[\\/]/)
  const filename = parts[parts.length - 1] || path

  // Remove common model file extensions
  return filename.replace(/\.(safetensors|ckpt|pt|pth|bin)$/i, '')
}

// Valid image path pattern: XX/YY/UUID.ext or XX/YY/UUID_thumb.webp
const VALID_PATH_PATTERN =
  /^[0-9a-f]{2}\/[0-9a-f]{2}\/[0-9a-f-]+(?:_thumb)?\.(?:png|jpg|jpeg|webp|gif)$/i

/**
 * Validates that an image path follows the expected format.
 * Returns the path if valid, or null if invalid.
 */
export function validateImagePath(path: string | null | undefined): string | null {
  if (!path) return null

  // Remove any leading slashes for validation
  const normalizedPath = path.replace(/^\/+/, '')

  // Check against pattern
  if (!VALID_PATH_PATTERN.test(normalizedPath)) {
    console.warn('Invalid image path format:', path)
    return null
  }

  // Ensure no path traversal attempts
  if (path.includes('..') || path.includes('//')) {
    console.warn('Path traversal attempt detected:', path)
    return null
  }

  return normalizedPath
}

/**
 * Builds a full image URL from a storage path.
 * Returns a placeholder if the path is invalid.
 */
export function getImageUrl(storagePath: string | null | undefined): string {
  const validPath = validateImagePath(storagePath)
  if (!validPath) {
    return '/placeholder-image.png'
  }
  return `/storage/${validPath}`
}

/**
 * Builds a full thumbnail URL from a thumbnail path.
 * Returns a placeholder if the path is invalid.
 */
export function getThumbnailUrl(thumbnailPath: string | null | undefined): string {
  const validPath = validateImagePath(thumbnailPath)
  if (!validPath) {
    return '/placeholder-thumbnail.png'
  }
  return `/storage/${validPath}`
}
