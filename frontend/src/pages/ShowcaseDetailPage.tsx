import { useState } from 'react'
import { useParams, useNavigate, Link } from 'react-router-dom'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { ArrowLeft, Image as ImageIcon, X, Play, ArrowUpDown } from 'lucide-react'
import toast from 'react-hot-toast'
import { showcasesApi } from '@/api/showcases'
import Slideshow from '@/components/gallery/Slideshow'
import SortableImageGrid from '@/components/gallery/SortableImageGrid'
import type { ShowcaseImageInfo } from '@/types/showcase'

export default function ShowcaseDetailPage() {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const [selectedImages, setSelectedImages] = useState<Set<string>>(new Set())
  const [isSelectMode, setIsSelectMode] = useState(false)
  const [isSortMode, setIsSortMode] = useState(false)
  const [slideshowStartIndex, setSlideshowStartIndex] = useState<number | null>(null)

  const {
    data: showcase,
    isLoading,
    error,
  } = useQuery({
    queryKey: ['showcase', id],
    queryFn: () => showcasesApi.get(id!),
    enabled: !!id,
  })

  const removeImagesMutation = useMutation({
    mutationFn: (imageIds: string[]) => showcasesApi.removeImages(id!, { image_ids: imageIds }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['showcase', id] })
      queryClient.invalidateQueries({ queryKey: ['showcases'] })
      toast.success('画像を削除しました')
      setSelectedImages(new Set())
      setIsSelectMode(false)
    },
    onError: () => {
      toast.error('画像の削除に失敗しました')
    },
  })

  const reorderImagesMutation = useMutation({
    mutationFn: (imageIds: string[]) => showcasesApi.reorderImages(id!, { image_ids: imageIds }),
    onError: () => {
      // Revert on error by refetching
      queryClient.invalidateQueries({ queryKey: ['showcase', id] })
      toast.error('並び替えの保存に失敗しました')
    },
  })

  const handleReorder = (newOrder: ShowcaseImageInfo[]) => {
    if (!showcase) return

    // Optimistic update
    queryClient.setQueryData(['showcase', id], {
      ...showcase,
      images: newOrder,
    })

    // Save to server
    reorderImagesMutation.mutate(newOrder.map((img) => img.id))
  }

  const handleImageClick = (imageId: string) => {
    if (isSelectMode) {
      setSelectedImages((prev) => {
        const next = new Set(prev)
        if (next.has(imageId)) {
          next.delete(imageId)
        } else {
          next.add(imageId)
        }
        return next
      })
    } else {
      navigate(`/image/${imageId}`)
    }
  }

  const handleRemoveSelected = () => {
    if (selectedImages.size === 0) return
    if (
      confirm(
        `選択した ${selectedImages.size} 枚の画像をShowcaseから削除しますか？\n画像自体は削除されません。`
      )
    ) {
      removeImagesMutation.mutate(Array.from(selectedImages))
    }
  }

  const toggleSelectMode = () => {
    setIsSelectMode(!isSelectMode)
    setIsSortMode(false)
    setSelectedImages(new Set())
  }

  const toggleSortMode = () => {
    setIsSortMode(!isSortMode)
    setIsSelectMode(false)
    setSelectedImages(new Set())
  }

  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-16">
        <div className="text-gray-400">Loading...</div>
      </div>
    )
  }

  if (error || !showcase) {
    return (
      <div className="flex flex-col items-center justify-center py-16">
        <div className="text-red-400 mb-4">Showcaseが見つかりません</div>
        <Link to="/showcases" className="text-blue-400 hover:underline">
          Showcase一覧に戻る
        </Link>
      </div>
    )
  }

  return (
    <div>
      {/* Header */}
      <div className="flex items-center gap-4 mb-6">
        <button
          onClick={() => navigate('/showcases')}
          className="p-2 hover:bg-gray-700 rounded-lg transition-colors"
          title="戻る"
        >
          <ArrowLeft size={20} />
        </button>
        <div className="flex-1">
          <h1 className="text-2xl font-bold">{showcase.name}</h1>
          {showcase.description && (
            <p className="text-gray-400 text-sm mt-1">{showcase.description}</p>
          )}
        </div>
        <div className="flex items-center gap-2">
          <span className="text-gray-400 text-sm">{showcase.image_count} 枚</span>
          {showcase.images.length > 0 && (
            <>
              <button
                onClick={() => setSlideshowStartIndex(0)}
                className="flex items-center gap-1.5 px-3 py-1.5 bg-green-600 rounded-lg text-sm hover:bg-green-700 transition-colors"
                title="スライドショー"
              >
                <Play size={16} />
                <span className="hidden sm:inline">スライドショー</span>
              </button>
              <button
                onClick={toggleSortMode}
                className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-sm transition-colors ${
                  isSortMode
                    ? 'bg-blue-600 text-white'
                    : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
                }`}
                title="並び替え"
              >
                <ArrowUpDown size={16} />
                <span className="hidden sm:inline">{isSortMode ? '完了' : '並び替え'}</span>
              </button>
            </>
          )}
          <button
            onClick={toggleSelectMode}
            className={`px-3 py-1.5 rounded-lg text-sm transition-colors ${
              isSelectMode
                ? 'bg-blue-600 text-white'
                : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
            }`}
          >
            {isSelectMode ? 'キャンセル' : '選択'}
          </button>
        </div>
      </div>

      {/* Selection toolbar */}
      {isSelectMode && selectedImages.size > 0 && (
        <div className="fixed bottom-4 left-1/2 -translate-x-1/2 bg-gray-800 border border-gray-700 rounded-lg px-4 py-3 shadow-xl flex items-center gap-4 z-40">
          <span className="text-sm">{selectedImages.size} 枚選択中</span>
          <button
            onClick={handleRemoveSelected}
            disabled={removeImagesMutation.isPending}
            className="flex items-center gap-2 px-3 py-1.5 bg-red-600 rounded-lg hover:bg-red-700 transition-colors text-sm disabled:opacity-50"
          >
            <X size={16} />
            Showcaseから削除
          </button>
        </div>
      )}

      {/* Images grid */}
      {showcase.images.length === 0 ? (
        <div className="text-center py-16">
          <ImageIcon size={64} className="mx-auto text-gray-600 mb-4" />
          <p className="text-gray-400 mb-4">このShowcaseにはまだ画像がありません</p>
          <p className="text-gray-500 text-sm">
            ギャラリーで画像を選択し、「Showcaseに追加」から追加できます
          </p>
        </div>
      ) : isSortMode ? (
        <SortableImageGrid images={showcase.images} onReorder={handleReorder} />
      ) : (
        <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 2xl:grid-cols-7 3xl:grid-cols-8 4xl:grid-cols-10 gap-2">
          {showcase.images.map((image) => (
            <div
              key={image.id}
              onClick={() => handleImageClick(image.id)}
              className={`relative aspect-square cursor-pointer rounded-lg overflow-hidden group ${
                isSelectMode && selectedImages.has(image.id) ? 'ring-2 ring-blue-500' : ''
              }`}
            >
              <img
                src={`/storage/${image.thumbnail_path}`}
                alt=""
                className="w-full h-full object-cover"
                loading="lazy"
              />
              {/* Selection indicator */}
              {isSelectMode && (
                <div
                  className={`absolute top-2 left-2 w-6 h-6 rounded-full border-2 flex items-center justify-center transition-colors ${
                    selectedImages.has(image.id)
                      ? 'bg-blue-500 border-blue-500'
                      : 'border-white/70 bg-black/30'
                  }`}
                >
                  {selectedImages.has(image.id) && (
                    <svg className="w-4 h-4 text-white" fill="currentColor" viewBox="0 0 20 20">
                      <path
                        fillRule="evenodd"
                        d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
                        clipRule="evenodd"
                      />
                    </svg>
                  )}
                </div>
              )}
              {/* Hover overlay */}
              {!isSelectMode && (
                <div className="absolute inset-0 bg-black/0 group-hover:bg-black/20 transition-colors" />
              )}
            </div>
          ))}
        </div>
      )}

      {/* Slideshow modal */}
      {slideshowStartIndex !== null && showcase.images.length > 0 && (
        <Slideshow
          images={showcase.images}
          startIndex={slideshowStartIndex}
          onClose={() => setSlideshowStartIndex(null)}
        />
      )}
    </div>
  )
}
