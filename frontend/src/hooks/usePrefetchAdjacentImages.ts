import { useEffect } from 'react'
import { useQueryClient } from '@tanstack/react-query'
import { imageDetailQueryOptions } from '@/api/images'
import { getImageUrl } from '@/utils/imagePath'
import { prefetchImage } from '@/utils/prefetch'
import type { Image } from '@/types/image'

/**
 * 詳細画面表示後に prev/next のメタデータと画像本体を先読みする。
 * 200ms のデバウンスにより、矢印キー連打で高速通過中の画像では発火しない。
 * ensureQueryData はキャッシュ済みなら再取得しないため、行き来しても無駄な取得は発生しない。
 */
export function usePrefetchAdjacentImages(
  image: Pick<Image, 'id' | 'prev_id' | 'next_id'> | undefined,
  locationSearch: string
): void {
  const queryClient = useQueryClient()
  const imageId = image?.id
  const prevId = image?.prev_id ?? null
  const nextId = image?.next_id ?? null

  useEffect(() => {
    if (!imageId) return
    // → キーでの順送りが主要動線のため next を先に読む
    const ids = [nextId, prevId].filter((v): v is string => v !== null)
    if (ids.length === 0) return

    const timer = window.setTimeout(() => {
      for (const adjId of ids) {
        queryClient
          .ensureQueryData(imageDetailQueryOptions(adjId, locationSearch))
          .then((adj) => prefetchImage(getImageUrl(adj.storage_path)))
          .catch(() => {
            // 先読み失敗は無視（本遷移時に通常フローで再取得される）
          })
      }
    }, 200)
    return () => window.clearTimeout(timer)
  }, [imageId, prevId, nextId, locationSearch, queryClient])
}
