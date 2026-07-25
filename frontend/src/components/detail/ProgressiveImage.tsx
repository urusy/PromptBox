import { useState, useCallback } from 'react'
import clsx from 'clsx'

interface ProgressiveImageProps {
  /** フル解像度画像の URL */
  src: string
  /** 先に表示するサムネイルの URL */
  placeholderSrc: string
  alt: string
  width: number
  height: number
  className?: string
  onClick?: () => void
  title?: string
}

/**
 * サムネイルを即時表示し、フル解像度画像のロード完了後にフェードで差し替える。
 * コンテナの aspect-ratio を元画像の寸法で確保するためレイアウトシフトしない。
 * 呼び出し側は画像切り替え時に key={image.id} を指定して強制リマウントすること。
 */
export default function ProgressiveImage({
  src,
  placeholderSrc,
  alt,
  width,
  height,
  className,
  onClick,
  title,
}: ProgressiveImageProps) {
  const [isLoaded, setIsLoaded] = useState(false)

  // ブラウザキャッシュから同期的にロードされると onLoad が発火しないことがある
  const handleImgRef = useCallback((node: HTMLImageElement | null) => {
    if (node && node.complete && node.naturalWidth > 0) {
      setIsLoaded(true)
    }
  }, [])

  return (
    <div
      className={clsx('relative overflow-hidden', className)}
      style={{ aspectRatio: `${width} / ${height}` }}
      onClick={onClick}
      title={title}
    >
      <img
        src={placeholderSrc}
        alt=""
        aria-hidden="true"
        decoding="async"
        className={clsx(
          'absolute inset-0 h-full w-full object-cover blur-sm scale-105 pointer-events-none transition-opacity duration-300',
          isLoaded ? 'opacity-0' : 'opacity-100'
        )}
      />
      <img
        ref={handleImgRef}
        src={src}
        alt={alt}
        width={width}
        height={height}
        decoding="async"
        fetchpriority="high"
        onLoad={() => setIsLoaded(true)}
        className={clsx(
          'h-full w-full object-contain transition-opacity duration-300',
          isLoaded ? 'opacity-100' : 'opacity-0'
        )}
      />
    </div>
  )
}
