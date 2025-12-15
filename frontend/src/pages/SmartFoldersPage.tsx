import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { FolderSearch, Plus, Pencil, Trash2, Star, Heart, Grid, Sparkles } from 'lucide-react'
import toast from 'react-hot-toast'
import { smartFoldersApi } from '@/api/smartFolders'
import { useConfirmDialog } from '@/hooks/useConfirmDialog'
import type { SmartFolder, SmartFolderCreate } from '@/types/smartFolder'
import type { SearchFilters } from '@/types/searchPreset'
import { filtersToSearchParams } from '@/utils/searchParams'

// Available icons for smart folders
const FOLDER_ICONS = [
  { value: 'folder', label: 'Folder', icon: FolderSearch },
  { value: 'star', label: 'Star', icon: Star },
  { value: 'heart', label: 'Heart', icon: Heart },
  { value: 'grid', label: 'Grid', icon: Grid },
  { value: 'sparkles', label: 'Sparkles', icon: Sparkles },
]

function getIconComponent(iconName: string | null) {
  const found = FOLDER_ICONS.find((i) => i.value === iconName)
  return found?.icon || FolderSearch
}

// Helper to get human-readable filter description
function getFilterDescription(filters: SearchFilters): string {
  const parts: string[] = []

  if (filters.q) parts.push(`"${filters.q}"`)
  if (filters.source_tool) parts.push(filters.source_tool.toUpperCase())
  if (filters.model_type) parts.push(filters.model_type.toUpperCase())
  if (filters.model_name) parts.push(`Model: ${filters.model_name}`)
  if (filters.min_rating !== undefined) parts.push(`Rating >= ${filters.min_rating}`)
  if (filters.exact_rating !== undefined) parts.push(`Rating = ${filters.exact_rating}`)
  if (filters.is_favorite) parts.push('Favorites')
  if (filters.needs_improvement) parts.push('Needs improvement')
  if (filters.is_xyz_grid === true) parts.push('Grid only')
  if (filters.is_xyz_grid === false) parts.push('Non-grid')
  if (filters.is_upscaled === true) parts.push('Upscaled')
  if (filters.is_upscaled === false) parts.push('Non-upscaled')
  if (filters.min_width) parts.push(`Width >= ${filters.min_width}px`)
  if (filters.min_height) parts.push(`Height >= ${filters.min_height}px`)
  if (filters.tags && filters.tags.length > 0) parts.push(`Tags: ${filters.tags.join(', ')}`)
  if (filters.lora_name) parts.push(`LoRA: ${filters.lora_name}`)

  return parts.length > 0 ? parts.join(' / ') : 'All images'
}

interface CreateEditModalProps {
  folder?: SmartFolder | null
  onClose: () => void
  onSave: (data: SmartFolderCreate) => void
  isPending: boolean
}

function CreateEditModal({ folder, onClose, onSave, isPending }: CreateEditModalProps) {
  const [name, setName] = useState(folder?.name || '')
  const [icon, setIcon] = useState(folder?.icon || 'folder')
  const [filters, setFilters] = useState<SearchFilters>(folder?.filters || {})

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (!name.trim()) {
      toast.error('名前を入力してください')
      return
    }
    onSave({ name: name.trim(), icon, filters })
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
          {folder ? 'スマートフォルダを編集' : '新しいスマートフォルダ'}
        </h2>

        <form onSubmit={handleSubmit} className="space-y-4">
          {/* Name */}
          <div>
            <label className="block text-sm text-gray-400 mb-1">名前</label>
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="フォルダ名"
              className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
              autoFocus
            />
          </div>

          {/* Icon */}
          <div>
            <label className="block text-sm text-gray-400 mb-1">アイコン</label>
            <div className="flex gap-2">
              {FOLDER_ICONS.map((item) => {
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

          {/* Filters */}
          <div className="space-y-3">
            <label className="block text-sm text-gray-400">フィルター条件</label>

            {/* Search query */}
            <input
              type="text"
              value={filters.q || ''}
              onChange={(e) => setFilters({ ...filters, q: e.target.value || undefined })}
              placeholder="検索キーワード"
              className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
            />

            {/* Model type */}
            <select
              value={filters.model_type || ''}
              onChange={(e) => setFilters({ ...filters, model_type: e.target.value || undefined })}
              className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              <option value="">Model Type: All</option>
              {['sd15', 'sdxl', 'pony', 'illustrious', 'flux', 'qwen'].map((t) => (
                <option key={t} value={t}>
                  {t.toUpperCase()}
                </option>
              ))}
            </select>

            {/* Source tool */}
            <select
              value={filters.source_tool || ''}
              onChange={(e) => setFilters({ ...filters, source_tool: e.target.value || undefined })}
              className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              <option value="">Source Tool: All</option>
              {['comfyui', 'a1111', 'forge', 'novelai'].map((t) => (
                <option key={t} value={t}>
                  {t.toUpperCase()}
                </option>
              ))}
            </select>

            {/* Rating */}
            <div className="flex gap-2">
              <select
                value={filters.exact_rating ?? filters.min_rating ?? ''}
                onChange={(e) => {
                  const val = e.target.value ? parseInt(e.target.value) : undefined
                  const isExactMode = filters.exact_rating !== undefined
                  if (isExactMode) {
                    setFilters({ ...filters, exact_rating: val, min_rating: undefined })
                  } else {
                    setFilters({ ...filters, min_rating: val, exact_rating: undefined })
                  }
                }}
                className="flex-1 px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
              >
                <option value="">Rating: Any</option>
                <option value="0">★0</option>
                {[1, 2, 3, 4, 5].map((r) => (
                  <option key={r} value={r}>
                    ★{r}
                  </option>
                ))}
              </select>
              <select
                value={filters.exact_rating !== undefined ? 'exact' : 'min'}
                onChange={(e) => {
                  const currentVal = filters.exact_rating ?? filters.min_rating
                  if (e.target.value === 'exact') {
                    setFilters({ ...filters, exact_rating: currentVal, min_rating: undefined })
                  } else {
                    setFilters({ ...filters, min_rating: currentVal, exact_rating: undefined })
                  }
                }}
                className="w-24 px-2 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
              >
                <option value="min">以上</option>
                <option value="exact">同等</option>
              </select>
            </div>

            {/* Checkboxes */}
            <div className="flex flex-wrap gap-4">
              <label className="flex items-center gap-2 cursor-pointer">
                <input
                  type="checkbox"
                  checked={filters.is_favorite === true}
                  onChange={(e) =>
                    setFilters({ ...filters, is_favorite: e.target.checked ? true : undefined })
                  }
                  className="w-4 h-4 rounded border-gray-600 bg-gray-700 text-blue-600 focus:ring-blue-500"
                />
                <span className="text-sm text-gray-300">Favorites</span>
              </label>
              <label className="flex items-center gap-2 cursor-pointer">
                <input
                  type="checkbox"
                  checked={filters.is_upscaled === true}
                  onChange={(e) =>
                    setFilters({ ...filters, is_upscaled: e.target.checked ? true : undefined })
                  }
                  className="w-4 h-4 rounded border-gray-600 bg-gray-700 text-blue-600 focus:ring-blue-500"
                />
                <span className="text-sm text-gray-300">Upscaled</span>
              </label>
            </div>
          </div>

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
              {folder ? '更新' : '作成'}
            </button>
          </div>
        </form>
      </div>
    </div>
  )
}

export default function SmartFoldersPage() {
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const { confirm, ConfirmDialogComponent } = useConfirmDialog()
  const [showModal, setShowModal] = useState(false)
  const [editingFolder, setEditingFolder] = useState<SmartFolder | null>(null)

  const { data: folders = [], isLoading } = useQuery({
    queryKey: ['smart-folders'],
    queryFn: smartFoldersApi.list,
  })

  const createMutation = useMutation({
    mutationFn: smartFoldersApi.create,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['smart-folders'] })
      toast.success('スマートフォルダを作成しました')
      setShowModal(false)
    },
    onError: () => {
      toast.error('スマートフォルダの作成に失敗しました')
    },
  })

  const updateMutation = useMutation({
    mutationFn: ({ id, data }: { id: string; data: SmartFolderCreate }) =>
      smartFoldersApi.update(id, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['smart-folders'] })
      toast.success('スマートフォルダを更新しました')
      setEditingFolder(null)
    },
    onError: () => {
      toast.error('スマートフォルダの更新に失敗しました')
    },
  })

  const deleteMutation = useMutation({
    mutationFn: smartFoldersApi.delete,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['smart-folders'] })
      toast.success('スマートフォルダを削除しました')
    },
    onError: () => {
      toast.error('スマートフォルダの削除に失敗しました')
    },
  })

  const handleFolderClick = (folder: SmartFolder) => {
    const searchParams = filtersToSearchParams(folder.filters)
    navigate(`/?${searchParams.toString()}`)
  }

  const handleEdit = (e: React.MouseEvent, folder: SmartFolder) => {
    e.stopPropagation()
    setEditingFolder(folder)
  }

  const handleDelete = async (e: React.MouseEvent, folder: SmartFolder) => {
    e.stopPropagation()
    const confirmed = await confirm({
      title: 'スマートフォルダを削除',
      message: `"${folder.name}" を削除しますか？`,
      confirmLabel: '削除',
      cancelLabel: 'キャンセル',
      variant: 'warning',
    })
    if (confirmed) {
      deleteMutation.mutate(folder.id)
    }
  }

  const handleSave = (data: SmartFolderCreate) => {
    if (editingFolder) {
      updateMutation.mutate({ id: editingFolder.id, data })
    } else {
      createMutation.mutate(data)
    }
  }

  const handleCloseModal = () => {
    setShowModal(false)
    setEditingFolder(null)
  }

  return (
    <>
      {ConfirmDialogComponent}
      <div>
        <div className="flex items-center justify-between mb-6">
          <h1 className="text-2xl font-bold">Smart Folders</h1>
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
      ) : folders.length === 0 ? (
        <div className="text-center py-16">
          <FolderSearch size={64} className="mx-auto text-gray-600 mb-4" />
          <p className="text-gray-400 mb-4">スマートフォルダがありません</p>
          <button
            onClick={() => setShowModal(true)}
            className="px-4 py-2 bg-blue-600 rounded-lg hover:bg-blue-700 transition-colors"
          >
            最初のスマートフォルダを作成
          </button>
        </div>
      ) : (
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 3xl:grid-cols-5 4xl:grid-cols-6 gap-4">
          {folders.map((folder) => {
            const IconComp = getIconComponent(folder.icon)
            return (
              <div
                key={folder.id}
                onClick={() => handleFolderClick(folder)}
                className="bg-gray-800 rounded-lg p-4 cursor-pointer hover:bg-gray-750 transition-colors group border border-gray-700 hover:border-gray-600"
              >
                <div className="flex items-start gap-3">
                  <div className="p-2 bg-blue-600/20 rounded-lg text-blue-400">
                    <IconComp size={24} />
                  </div>
                  <div className="flex-1 min-w-0">
                    <h3 className="font-medium text-white truncate">{folder.name}</h3>
                    <p className="text-sm text-gray-400 mt-1 line-clamp-2">
                      {getFilterDescription(folder.filters)}
                    </p>
                  </div>
                  <div className="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                    <button
                      onClick={(e) => handleEdit(e, folder)}
                      className="p-1.5 text-gray-400 hover:text-white hover:bg-gray-700 rounded"
                      title="編集"
                    >
                      <Pencil size={16} />
                    </button>
                    <button
                      onClick={(e) => handleDelete(e, folder)}
                      className="p-1.5 text-gray-400 hover:text-red-500 hover:bg-gray-700 rounded"
                      title="削除"
                    >
                      <Trash2 size={16} />
                    </button>
                  </div>
                </div>
              </div>
            )
          })}
        </div>
      )}

        {/* Create/Edit Modal */}
        {(showModal || editingFolder) && (
          <CreateEditModal
            folder={editingFolder}
            onClose={handleCloseModal}
            onSave={handleSave}
            isPending={createMutation.isPending || updateMutation.isPending}
          />
        )}
      </div>
    </>
  )
}
