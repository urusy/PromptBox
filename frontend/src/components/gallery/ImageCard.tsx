import { memo, useCallback } from 'react'
import { Link, useLocation } from 'react-router-dom'
import { Star, Heart, Check } from 'lucide-react'
import clsx from 'clsx'
import type { ImageListItem } from '@/types/image'
import { useSelectionStore } from '@/stores/selectionStore'

interface ImageCardProps {
  image: ImageListItem
}

const ImageCard = memo(function ImageCard({ image }: ImageCardProps) {
  const location = useLocation()
  const { selectedIds, isSelectionMode, toggleSelection, setSelectionMode } = useSelectionStore()
  const isSelected = selectedIds.has(image.id)

  const handleClick = useCallback((e: React.MouseEvent) => {
    if (isSelectionMode) {
      e.preventDefault()
      toggleSelection(image.id)
    }
  }, [isSelectionMode, toggleSelection, image.id])

  const handleLongPress = useCallback((e: React.MouseEvent) => {
    // Enable selection mode on right-click or ctrl+click
    if (e.ctrlKey || e.metaKey) {
      e.preventDefault()
      if (!isSelectionMode) {
        setSelectionMode(true)
      }
      toggleSelection(image.id)
    }
  }, [isSelectionMode, setSelectionMode, toggleSelection, image.id])

  return (
    <Link
      to={isSelectionMode ? '#' : `/image/${image.id}${location.search}`}
      onClick={handleClick}
      onMouseDown={handleLongPress}
      className={clsx(
        'group relative bg-gray-800 rounded-lg overflow-hidden transition-all',
        isSelected
          ? 'ring-2 ring-blue-500 ring-offset-2 ring-offset-gray-900'
          : 'hover:ring-2 hover:ring-blue-500/50'
      )}
    >
      <div className="aspect-square">
        <img
          src={`/storage/${image.thumbnail_path}`}
          alt={image.model_name || 'Generated image'}
          className={clsx(
            'w-full h-full object-cover transition-opacity',
            isSelected && 'opacity-75'
          )}
          loading="lazy"
        />
      </div>

      {/* Selection checkbox */}
      {(isSelectionMode || isSelected) && (
        <div
          className={clsx(
            'absolute top-2 left-2 w-6 h-6 rounded-full flex items-center justify-center transition-colors',
            isSelected
              ? 'bg-blue-500 text-white'
              : 'bg-black/50 text-white/50 group-hover:bg-black/70'
          )}
        >
          {isSelected && <Check size={14} />}
        </div>
      )}

      {/* Overlay on hover */}
      <div className="absolute inset-0 bg-gradient-to-t from-black/70 via-transparent to-transparent opacity-0 group-hover:opacity-100 transition-opacity">
        <div className="absolute bottom-0 left-0 right-0 p-3">
          <p className="text-sm text-white truncate font-medium">
            {image.model_name || 'Unknown model'}
          </p>
          <p className="text-xs text-gray-300 mt-0.5">
            {image.source_tool.toUpperCase()}
            {image.model_type && ` • ${image.model_type.toUpperCase()}`}
          </p>
        </div>
      </div>

      {/* Badges */}
      <div className="absolute top-2 right-2 flex items-center gap-1">
        {image.rating > 0 && (
          <div className="flex items-center gap-0.5 px-1.5 py-0.5 bg-black/60 rounded text-yellow-400">
            <Star size={12} fill="currentColor" />
            <span className="text-xs">{image.rating}</span>
          </div>
        )}
        {image.is_favorite && (
          <div className="p-1 bg-black/60 rounded">
            <Heart size={12} className="text-red-500" fill="currentColor" />
          </div>
        )}
      </div>
    </Link>
  )
})

export default ImageCard
