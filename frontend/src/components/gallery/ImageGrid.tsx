import { useRef, useCallback, useState, useEffect } from 'react'
import { useVirtualizer } from '@tanstack/react-virtual'
import type { ImageListItem } from '@/types/image'
import { useColumnCount } from '@/hooks/useColumnCount'
import { useContainerWidth } from '@/hooks/useContainerWidth'
import { distributeWaterfall, computeJustifiedRows, getAspectRatio } from '@/utils/galleryLayout'
import ImageCard from './ImageCard'

export type GridSize = 'small' | 'medium' | 'large'
export type GalleryLayout = 'square' | 'justified' | 'waterfall'

interface ImageGridProps {
  images: ImageListItem[]
  size?: GridSize
  layout?: GalleryLayout
  /**
   * カードの遷移先ベースパス。グリッド専用一覧（/grids）から使うときは
   * グリッド詳細へ飛ばすため '/grids' を渡す。
   */
  linkBase?: string
}

// Gap between items (gap-3 = 12px)
const ROW_GAP = 12

// Row height estimates based on grid size (including gap)
const ROW_HEIGHT: Record<GridSize, number> = {
  small: 140 + ROW_GAP,
  medium: 200 + ROW_GAP,
  large: 280 + ROW_GAP,
}

// Target row height (Justified) / target thumbnail height by grid size
const TARGET_HEIGHT: Record<GridSize, number> = {
  small: 140,
  medium: 200,
  large: 280,
}

// Threshold for enabling virtual scrolling (square layout only)
const VIRTUAL_SCROLL_THRESHOLD = 100

interface SubGridProps {
  images: ImageListItem[]
  size: GridSize
  linkBase?: string
}

function EmptyState() {
  return (
    <div className="text-center text-gray-400 py-16">
      <p className="text-xl">No images found</p>
      <p className="mt-2">Try adjusting your search filters</p>
    </div>
  )
}

export default function ImageGrid({
  images,
  size = 'medium',
  layout = 'square',
  linkBase,
}: ImageGridProps) {
  if (images.length === 0) {
    return <EmptyState />
  }

  if (layout === 'waterfall') {
    return <WaterfallGrid images={images} size={size} linkBase={linkBase} />
  }

  if (layout === 'justified') {
    return <JustifiedGrid images={images} size={size} linkBase={linkBase} />
  }

  return <SquareGrid images={images} size={size} linkBase={linkBase} />
}

/**
 * 正方形グリッド（従来実装）。小規模は通常グリッド、100件以上で行仮想スクロール。
 */
function SquareGrid({ images, size, linkBase }: SubGridProps) {
  const parentRef = useRef<HTMLDivElement>(null)
  const columnCount = useColumnCount(size)
  const rowCount = Math.ceil(images.length / columnCount)
  const [containerHeight, setContainerHeight] = useState(0)

  // Calculate available height for virtual scroll container
  useEffect(() => {
    const calculateHeight = () => {
      const availableHeight = window.innerHeight - 280
      setContainerHeight(Math.max(400, availableHeight))
    }

    calculateHeight()
    window.addEventListener('resize', calculateHeight)
    return () => window.removeEventListener('resize', calculateHeight)
  }, [])

  const rowVirtualizer = useVirtualizer({
    count: rowCount,
    getScrollElement: () => parentRef.current,
    estimateSize: useCallback(() => ROW_HEIGHT[size], [size]),
    overscan: 3,
  })

  // Use simple grid for small datasets, virtual scroll for large ones
  if (images.length < VIRTUAL_SCROLL_THRESHOLD) {
    return (
      <div
        className="grid gap-3"
        style={{ gridTemplateColumns: `repeat(${columnCount}, minmax(0, 1fr))` }}
      >
        {images.map((image) => (
          <ImageCard key={image.id} image={image} linkBase={linkBase} />
        ))}
      </div>
    )
  }

  const virtualRows = rowVirtualizer.getVirtualItems()

  return (
    <div
      ref={parentRef}
      className="overflow-auto"
      style={{ height: containerHeight, contain: 'strict' }}
    >
      <div className="relative w-full" style={{ height: rowVirtualizer.getTotalSize() }}>
        {virtualRows.map((virtualRow) => {
          const startIndex = virtualRow.index * columnCount
          const rowImages = images.slice(startIndex, startIndex + columnCount)

          return (
            <div
              key={virtualRow.key}
              className="absolute left-0 right-0 grid gap-3"
              style={{
                top: virtualRow.start,
                height: virtualRow.size,
                gridTemplateColumns: `repeat(${columnCount}, minmax(0, 1fr))`,
              }}
            >
              {rowImages.map((image) => (
                <ImageCard key={image.id} image={image} linkBase={linkBase} />
              ))}
            </div>
          )
        })}
      </div>
    </div>
  )
}

/**
 * Waterfall（Pinterest 風 Masonry）。列幅固定・最短列へ順次配置。
 * アスペクト比を保持し、切り抜きなし。
 */
function WaterfallGrid({ images, size, linkBase }: SubGridProps) {
  const columnCount = useColumnCount(size)
  const columns = distributeWaterfall(images, columnCount)

  return (
    <div className="flex gap-3 items-start">
      {columns.map((column, i) => (
        <div key={i} className="flex flex-col gap-3 flex-1 min-w-0">
          {column.map((image) => (
            <ImageCard
              key={image.id}
              image={image}
              aspectRatio={getAspectRatio(image)}
              linkBase={linkBase}
            />
          ))}
        </div>
      ))}
    </div>
  )
}

/**
 * Justified rows（Google Photos 風）。各行を横幅いっぱいに揃え、
 * アスペクト比を保持したまま切り抜きなしで表示。
 */
function JustifiedGrid({ images, size, linkBase }: SubGridProps) {
  const [ref, width] = useContainerWidth<HTMLDivElement>()
  const rows = width > 0 ? computeJustifiedRows(images, width, TARGET_HEIGHT[size], ROW_GAP) : []

  return (
    <div ref={ref} className="flex flex-col gap-3">
      {rows.map((row, i) => (
        <div key={i} className="flex gap-3" style={{ height: row.rowHeight }}>
          {row.items.map(({ image, width: itemWidth }) => (
            <div key={image.id} style={{ width: itemWidth }} className="shrink-0">
              <ImageCard image={image} aspectRatio={getAspectRatio(image)} linkBase={linkBase} />
            </div>
          ))}
        </div>
      ))}
    </div>
  )
}
