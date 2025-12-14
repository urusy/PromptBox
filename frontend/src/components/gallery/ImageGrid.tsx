import { useRef, useCallback, useState, useEffect } from 'react'
import { useVirtualizer } from '@tanstack/react-virtual'
import type { ImageListItem } from '@/types/image'
import { useColumnCount } from '@/hooks/useColumnCount'
import ImageCard from './ImageCard'

export type GridSize = 'small' | 'medium' | 'large'

interface ImageGridProps {
  images: ImageListItem[]
  size?: GridSize
}

// Row height estimates based on grid size (including gap)
const ROW_HEIGHT: Record<GridSize, number> = {
  small: 140,
  medium: 200,
  large: 280,
}

// Threshold for enabling virtual scrolling
const VIRTUAL_SCROLL_THRESHOLD = 100

export default function ImageGrid({ images, size = 'medium' }: ImageGridProps) {
  const parentRef = useRef<HTMLDivElement>(null)
  const columnCount = useColumnCount(size)
  const rowCount = Math.ceil(images.length / columnCount)
  const [containerHeight, setContainerHeight] = useState(0)

  // Calculate available height for virtual scroll container
  useEffect(() => {
    const calculateHeight = () => {
      // Use viewport height minus header/toolbar space
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
    overscan: 3, // Render 3 extra rows above and below viewport
  })

  if (images.length === 0) {
    return (
      <div className="text-center text-gray-400 py-16">
        <p className="text-xl">No images found</p>
        <p className="mt-2">Try adjusting your search filters</p>
      </div>
    )
  }

  // Use simple grid for small datasets, virtual scroll for large ones
  if (images.length < VIRTUAL_SCROLL_THRESHOLD) {
    return (
      <div
        className="grid gap-3"
        style={{ gridTemplateColumns: `repeat(${columnCount}, minmax(0, 1fr))` }}
      >
        {images.map((image) => (
          <ImageCard key={image.id} image={image} />
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
                <ImageCard key={image.id} image={image} />
              ))}
            </div>
          )
        })}
      </div>
    </div>
  )
}
