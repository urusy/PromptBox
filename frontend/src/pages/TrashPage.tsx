import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { RotateCcw, Trash2, AlertTriangle } from 'lucide-react'
import toast from 'react-hot-toast'
import { imagesApi } from '@/api/images'
import { batchApi } from '@/api/batch'
import { useConfirmDialog } from '@/hooks/useConfirmDialog'
import type { ImageSearchParams } from '@/types/image'

export default function TrashPage() {
  const queryClient = useQueryClient()
  const { confirm, ConfirmDialogComponent } = useConfirmDialog()
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

  const emptyTrashMutation = useMutation({
    mutationFn: (ids: string[]) => batchApi.delete({ ids, permanent: true }),
    onSuccess: () => {
      toast.success('Trash emptied')
      queryClient.invalidateQueries({ queryKey: ['images'] })
    },
    onError: () => {
      toast.error('Failed to empty trash')
    },
  })

  const handleEmptyTrash = async () => {
    if (!data || data.items.length === 0) return
    const ids = data.items.map((img) => img.id)
    const confirmed = await confirm({
      title: 'ゴミ箱を空にする',
      message: `${ids.length}枚の画像を完全に削除しますか？\nこの操作は取り消せません。`,
      confirmLabel: '削除',
      cancelLabel: 'キャンセル',
      variant: 'danger',
    })
    if (confirmed) {
      emptyTrashMutation.mutate(ids)
    }
  }

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin rounded-full h-12 w-12 border-t-2 border-b-2 border-blue-500"></div>
      </div>
    )
  }

  if (error) {
    return (
      <div className="text-center text-red-500 py-8">Failed to load trash. Please try again.</div>
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
    <>
      {ConfirmDialogComponent}
      <div>
        <div className="flex items-center justify-between mb-6">
          <h1 className="text-2xl font-bold">Trash</h1>
        <button
          onClick={handleEmptyTrash}
          disabled={emptyTrashMutation.isPending}
          className="flex items-center gap-2 px-4 py-2 bg-red-600 hover:bg-red-700 disabled:bg-red-800 disabled:cursor-not-allowed rounded-lg transition-colors"
        >
          <AlertTriangle size={18} />
          {emptyTrashMutation.isPending ? 'Emptying...' : 'Empty Trash'}
          <span className="text-sm text-red-200">({data.items.length})</span>
        </button>
      </div>
      <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 2xl:grid-cols-6 3xl:grid-cols-8 4xl:grid-cols-10 5xl:grid-cols-12 gap-4">
        {data.items.map((image) => (
          <div key={image.id} className="group relative bg-gray-800 rounded-lg overflow-hidden">
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
                onClick={async () => {
                  const confirmed = await confirm({
                    title: '完全に削除',
                    message: 'この画像を完全に削除しますか？\nこの操作は取り消せません。',
                    confirmLabel: '削除',
                    cancelLabel: 'キャンセル',
                    variant: 'danger',
                  })
                  if (confirmed) {
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
    </>
  )
}
