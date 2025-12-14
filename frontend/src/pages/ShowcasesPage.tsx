import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { Album, Plus, Pencil, Trash2, Star, Heart, Grid, Sparkles, Image as ImageIcon, X } from 'lucide-react'
import toast from 'react-hot-toast'
import { showcasesApi } from '@/api/showcases'
import type { Showcase, ShowcaseCreate, ShowcaseUpdate, ShowcaseImageInfo } from '@/types/showcase'

// Available icons for showcases
const SHOWCASE_ICONS = [
  { value: 'album', label: 'Album', icon: Album },
  { value: 'star', label: 'Star', icon: Star },
  { value: 'heart', label: 'Heart', icon: Heart },
  { value: 'grid', label: 'Grid', icon: Grid },
  { value: 'sparkles', label: 'Sparkles', icon: Sparkles },
]

function getIconComponent(iconName: string | null) {
  const found = SHOWCASE_ICONS.find((i) => i.value === iconName)
  return found?.icon || Album
}

interface CreateEditModalProps {
  showcase?: Showcase | null
  showcaseImages?: ShowcaseImageInfo[]
  onClose: () => void
  onSave: (data: ShowcaseCreate | ShowcaseUpdate) => void
  isPending: boolean
}

function CreateEditModal({ showcase, showcaseImages, onClose, onSave, isPending }: CreateEditModalProps) {
  const [name, setName] = useState(showcase?.name || '')
  const [description, setDescription] = useState(showcase?.description || '')
  const [icon, setIcon] = useState(showcase?.icon || 'album')
  const [coverImageId, setCoverImageId] = useState<string | null>(showcase?.cover_image_id || null)

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (!name.trim()) {
      toast.error('名前を入力してください')
      return
    }
    if (showcase) {
      // Update
      onSave({
        name: name.trim(),
        description: description.trim() || null,
        icon,
        cover_image_id: coverImageId,
      } as ShowcaseUpdate)
    } else {
      // Create
      onSave({
        name: name.trim(),
        description: description.trim() || null,
        icon,
      } as ShowcaseCreate)
    }
  }

  return (
    <div
      className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4"
      onClick={onClose}
    >
      <div
        className="bg-gray-800 rounded-lg p-6 w-full max-w-lg max-h-[90vh] overflow-y-auto"
        onClick={(e) => e.stopPropagation()}
      >
        <h2 className="text-xl font-semibold text-white mb-4">
          {showcase ? 'Showcaseを編集' : '新しいShowcase'}
        </h2>

        <form onSubmit={handleSubmit} className="space-y-4">
          {/* Name */}
          <div>
            <label className="block text-sm text-gray-400 mb-1">名前</label>
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="Showcase名"
              className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
              autoFocus
            />
          </div>

          {/* Description */}
          <div>
            <label className="block text-sm text-gray-400 mb-1">説明（オプション）</label>
            <textarea
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder="Showcaseの説明"
              rows={2}
              className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 resize-none"
            />
          </div>

          {/* Icon */}
          <div>
            <label className="block text-sm text-gray-400 mb-1">アイコン</label>
            <div className="flex gap-2">
              {SHOWCASE_ICONS.map((item) => {
                const IconComp = item.icon
                return (
                  <button
                    key={item.value}
                    type="button"
                    onClick={() => setIcon(item.value)}
                    className={`p-2 rounded-lg transition-colors ${
                      icon === item.value
                        ? 'bg-blue-600 text-white'
                        : 'bg-gray-700 text-gray-400 hover:bg-gray-600'
                    }`}
                    title={item.label}
                  >
                    <IconComp size={20} />
                  </button>
                )
              })}
            </div>
          </div>

          {/* Cover Image (only when editing and has images) */}
          {showcase && showcaseImages && showcaseImages.length > 0 && (
            <div>
              <label className="block text-sm text-gray-400 mb-2">カバー画像</label>
              <div className="grid grid-cols-4 auto-rows-min gap-2 max-h-48 overflow-y-auto p-1">
                {/* None option */}
                <button
                  type="button"
                  onClick={() => setCoverImageId(null)}
                  className={`aspect-square rounded-lg flex items-center justify-center transition-all ${
                    coverImageId === null
                      ? 'ring-2 ring-blue-500 bg-gray-600'
                      : 'bg-gray-700 hover:bg-gray-600'
                  }`}
                  title="カバーなし"
                >
                  <X size={24} className="text-gray-400" />
                </button>
                {/* Image options */}
                {showcaseImages.map((image) => (
                  <button
                    key={image.id}
                    type="button"
                    onClick={() => setCoverImageId(image.id)}
                    className={`aspect-square rounded-lg overflow-hidden transition-all ${
                      coverImageId === image.id
                        ? 'ring-2 ring-blue-500'
                        : 'hover:ring-2 hover:ring-gray-500'
                    }`}
                  >
                    <img
                      src={`/storage/${image.thumbnail_path}`}
                      alt=""
                      className="w-full h-full object-cover"
                    />
                  </button>
                ))}
              </div>
            </div>
          )}

          {/* Actions */}
          <div className="flex gap-2 justify-end pt-4">
            <button
              type="button"
              onClick={onClose}
              className="px-4 py-2 text-gray-400 hover:text-white transition-colors"
            >
              キャンセル
            </button>
            <button
              type="submit"
              disabled={isPending}
              className="px-4 py-2 bg-blue-600 rounded-lg hover:bg-blue-700 transition-colors disabled:opacity-50"
            >
              {showcase ? '更新' : '作成'}
            </button>
          </div>
        </form>
      </div>
    </div>
  )
}

export default function ShowcasesPage() {
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const [showModal, setShowModal] = useState(false)
  const [editingShowcaseId, setEditingShowcaseId] = useState<string | null>(null)

  const { data: showcases = [], isLoading } = useQuery({
    queryKey: ['showcases'],
    queryFn: showcasesApi.list,
  })

  // Fetch showcase detail when editing (to get images for cover selection)
  const { data: editingShowcaseDetail } = useQuery({
    queryKey: ['showcase', editingShowcaseId],
    queryFn: () => showcasesApi.get(editingShowcaseId!),
    enabled: !!editingShowcaseId,
  })

  const createMutation = useMutation({
    mutationFn: showcasesApi.create,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['showcases'] })
      toast.success('Showcaseを作成しました')
      setShowModal(false)
    },
    onError: () => {
      toast.error('Showcaseの作成に失敗しました')
    },
  })

  const updateMutation = useMutation({
    mutationFn: ({ id, data }: { id: string; data: ShowcaseUpdate }) =>
      showcasesApi.update(id, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['showcases'] })
      queryClient.invalidateQueries({ queryKey: ['showcase', editingShowcaseId] })
      toast.success('Showcaseを更新しました')
      setEditingShowcaseId(null)
    },
    onError: () => {
      toast.error('Showcaseの更新に失敗しました')
    },
  })

  const deleteMutation = useMutation({
    mutationFn: showcasesApi.delete,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['showcases'] })
      toast.success('Showcaseを削除しました')
    },
    onError: () => {
      toast.error('Showcaseの削除に失敗しました')
    },
  })

  const handleShowcaseClick = (showcase: Showcase) => {
    navigate(`/showcase/${showcase.id}`)
  }

  const handleEdit = (e: React.MouseEvent, showcase: Showcase) => {
    e.stopPropagation()
    setEditingShowcaseId(showcase.id)
  }

  const handleDelete = (e: React.MouseEvent, showcase: Showcase) => {
    e.stopPropagation()
    if (confirm(`"${showcase.name}" を削除しますか？\n含まれる画像は削除されません。`)) {
      deleteMutation.mutate(showcase.id)
    }
  }

  const handleSave = (data: ShowcaseCreate | ShowcaseUpdate) => {
    if (editingShowcaseId) {
      updateMutation.mutate({ id: editingShowcaseId, data: data as ShowcaseUpdate })
    } else {
      createMutation.mutate(data as ShowcaseCreate)
    }
  }

  const handleCloseModal = () => {
    setShowModal(false)
    setEditingShowcaseId(null)
  }

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-2xl font-bold">Showcases</h1>
        <button
          onClick={() => setShowModal(true)}
          className="flex items-center gap-2 px-4 py-2 bg-blue-600 rounded-lg hover:bg-blue-700 transition-colors"
        >
          <Plus size={18} />
          <span className="hidden sm:inline">新規作成</span>
        </button>
      </div>

      {isLoading ? (
        <div className="text-center text-gray-400 py-8">Loading...</div>
      ) : showcases.length === 0 ? (
        <div className="text-center py-16">
          <Album size={64} className="mx-auto text-gray-600 mb-4" />
          <p className="text-gray-400 mb-4">Showcaseがありません</p>
          <button
            onClick={() => setShowModal(true)}
            className="px-4 py-2 bg-blue-600 rounded-lg hover:bg-blue-700 transition-colors"
          >
            最初のShowcaseを作成
          </button>
        </div>
      ) : (
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 3xl:grid-cols-5 4xl:grid-cols-6 gap-4">
          {showcases.map((showcase) => {
            const IconComp = getIconComponent(showcase.icon)
            return (
              <div
                key={showcase.id}
                onClick={() => handleShowcaseClick(showcase)}
                className="bg-gray-800 rounded-lg overflow-hidden cursor-pointer hover:bg-gray-750 transition-colors group border border-gray-700 hover:border-gray-600"
              >
                {/* Cover image or placeholder */}
                <div className="aspect-video bg-gray-700 relative">
                  {showcase.cover_thumbnail_path ? (
                    <img
                      src={`/storage/${showcase.cover_thumbnail_path}`}
                      alt=""
                      className="w-full h-full object-cover"
                    />
                  ) : (
                    <div className="w-full h-full flex items-center justify-center">
                      <ImageIcon size={48} className="text-gray-600" />
                    </div>
                  )}
                  {/* Image count badge */}
                  <div className="absolute bottom-2 right-2 bg-black/70 px-2 py-1 rounded text-sm">
                    {showcase.image_count} 枚
                  </div>
                </div>

                {/* Info */}
                <div className="p-4">
                  <div className="flex items-start gap-3">
                    <div className="p-2 bg-purple-600/20 rounded-lg text-purple-400 shrink-0">
                      <IconComp size={20} />
                    </div>
                    <div className="flex-1 min-w-0">
                      <h3 className="font-medium text-white truncate">{showcase.name}</h3>
                      {showcase.description && (
                        <p className="text-sm text-gray-400 mt-1 line-clamp-2">
                          {showcase.description}
                        </p>
                      )}
                    </div>
                    <div className="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity shrink-0">
                      <button
                        onClick={(e) => handleEdit(e, showcase)}
                        className="p-1.5 text-gray-400 hover:text-white hover:bg-gray-700 rounded"
                        title="編集"
                      >
                        <Pencil size={16} />
                      </button>
                      <button
                        onClick={(e) => handleDelete(e, showcase)}
                        className="p-1.5 text-gray-400 hover:text-red-500 hover:bg-gray-700 rounded"
                        title="削除"
                      >
                        <Trash2 size={16} />
                      </button>
                    </div>
                  </div>
                </div>
              </div>
            )
          })}
        </div>
      )}

      {/* Create/Edit Modal */}
      {(showModal || editingShowcaseId) && (
        <CreateEditModal
          showcase={editingShowcaseDetail || null}
          showcaseImages={editingShowcaseDetail?.images}
          onClose={handleCloseModal}
          onSave={handleSave}
          isPending={createMutation.isPending || updateMutation.isPending}
        />
      )}
    </div>
  )
}
