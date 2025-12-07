import { Link } from 'react-router-dom'
import { Star, Heart } from 'lucide-react'
import type { ImageListItem } from '@/types/image'

interface ImageCardProps {
  image: ImageListItem
}

export default function ImageCard({ image }: ImageCardProps) {
  return (
    <Link
      to={`/image/${image.id}`}
      className="group relative bg-gray-800 rounded-lg overflow-hidden hover:ring-2 hover:ring-blue-500 transition-all"
    >
      <div className="aspect-square">
        <img
          src={`/storage/${image.thumbnail_path}`}
          alt={image.model_name || 'Generated image'}
          className="w-full h-full object-cover"
          loading="lazy"
        />
      </div>

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
}
