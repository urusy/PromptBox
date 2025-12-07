import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { RotateCcw, Trash2 } from 'lucide-react'
import toast from 'react-hot-toast'
import { imagesApi } from '@/api/images'
import type { ImageSearchParams } from '@/types/image'

export default function TrashPage() {
  const queryClient = useQueryClient()
  const [params] = useState<ImageSearchParams>({
    page: 1,
    per_page: 24,
    include_deleted: true,
    sort_by: 'created_at',
    sort_order: 'desc',
  })

  const { data, isLoading, error } = useQuery({
    queryKey: ['images', 'trash', params],
    queryFn: () => imagesApi.list(params),
  })

  const restoreMutation = useMutation({
    mutationFn: (id: string) => imagesApi.restore(id),
    onSuccess: () => {
      toast.success('Image restored')
      queryClient.invalidateQueries({ queryKey: ['images'] })
    },
  })

  const deleteMutation = useMutation({
    mutationFn: (id: string) => imagesApi.delete(id, true),
    onSuccess: () => {
      toast.success('Image permanently deleted')
      queryClient.invalidateQueries({ queryKey: ['images'] })
    },
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
        Failed to load trash. Please try again.
      </div>
    )
  }

  if (!data || data.items.length === 0) {
    return (
      <div className="text-center text-gray-400 py-16">
        <Trash2 size={48} className="mx-auto mb-4" />
        <p className="text-xl">Trash is empty</p>
      </div>
    )
  }

  return (
    <div>
      <h1 className="text-2xl font-bold mb-6">Trash</h1>
      <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 2xl:grid-cols-6 gap-4">
        {data.items.map((image) => (
          <div
            key={image.id}
            className="group relative bg-gray-800 rounded-lg overflow-hidden"
          >
            <div className="aspect-square opacity-60">
              <img
                src={`/storage/${image.thumbnail_path}`}
                alt={image.model_name || 'Deleted image'}
                className="w-full h-full object-cover"
                loading="lazy"
              />
            </div>
            <div className="absolute inset-0 flex items-center justify-center gap-4 opacity-0 group-hover:opacity-100 transition-opacity bg-black/50">
              <button
                onClick={() => restoreMutation.mutate(image.id)}
                className="p-3 bg-green-600 rounded-full hover:bg-green-700"
                title="Restore"
              >
                <RotateCcw size={20} />
              </button>
              <button
                onClick={() => {
                  if (confirm('Permanently delete this image? This cannot be undone.')) {
                    deleteMutation.mutate(image.id)
                  }
                }}
                className="p-3 bg-red-600 rounded-full hover:bg-red-700"
                title="Delete permanently"
              >
                <Trash2 size={20} />
              </button>
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}
