import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { Link } from 'react-router-dom'
import { Star, Heart } from 'lucide-react'
import { imagesApi } from '@/api/images'
import type { ImageSearchParams } from '@/types/image'

export default function GalleryPage() {
  const [params, setParams] = useState<ImageSearchParams>({
    page: 1,
    per_page: 24,
    sort_by: 'created_at',
    sort_order: 'desc',
  })

  const { data, isLoading, error } = useQuery({
    queryKey: ['images', params],
    queryFn: () => imagesApi.list(params),
  })

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin rounded-full h-12 w-12 border-t-2 border-b-2 border-blue-500"></div>
      </div>
    )
  }

  if (error) {
    return (
      <div className="text-center text-red-500 py-8">
        Failed to load images. Please try again.
      </div>
    )
  }

  if (!data || data.items.length === 0) {
    return (
      <div className="text-center text-gray-400 py-16">
        <p className="text-xl">No images found</p>
        <p className="mt-2">Add images to the import folder to get started</p>
      </div>
    )
  }

  return (
    <div>
      <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 2xl:grid-cols-6 gap-4">
        {data.items.map((image) => (
          <Link
            key={image.id}
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
            <div className="absolute inset-0 bg-gradient-to-t from-black/60 to-transparent opacity-0 group-hover:opacity-100 transition-opacity">
              <div className="absolute bottom-0 left-0 right-0 p-3">
                <p className="text-sm text-white truncate">
                  {image.model_name || 'Unknown model'}
                </p>
                <div className="flex items-center gap-2 mt-1">
                  {image.rating > 0 && (
                    <div className="flex items-center gap-1 text-yellow-400">
                      <Star size={14} fill="currentColor" />
                      <span className="text-xs">{image.rating}</span>
                    </div>
                  )}
                  {image.is_favorite && (
                    <Heart size={14} className="text-red-500" fill="currentColor" />
                  )}
                </div>
              </div>
            </div>
          </Link>
        ))}
      </div>

      {data.total_pages > 1 && (
        <div className="flex items-center justify-center gap-2 mt-8">
          <button
            onClick={() => setParams({ ...params, page: params.page! - 1 })}
            disabled={params.page === 1}
            className="px-4 py-2 bg-gray-800 rounded-md hover:bg-gray-700 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            Previous
          </button>
          <span className="px-4 py-2 text-gray-400">
            Page {params.page} of {data.total_pages}
          </span>
          <button
            onClick={() => setParams({ ...params, page: params.page! + 1 })}
            disabled={params.page === data.total_pages}
            className="px-4 py-2 bg-gray-800 rounded-md hover:bg-gray-700 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            Next
          </button>
        </div>
      )}
    </div>
  )
}
