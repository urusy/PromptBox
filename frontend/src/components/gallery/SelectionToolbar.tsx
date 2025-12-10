import { useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { X, Star, Heart, Trash2, Tag, CheckSquare, Square, Download } from 'lucide-react'
import toast from 'react-hot-toast'
import { batchApi } from '@/api/batch'
import { useSelectionStore } from '@/stores/selectionStore'
import { RATING_LABELS } from '@/components/common/StarRating'

interface SelectionToolbarProps {
  totalCount: number
  allIds: string[]
  isSelectionMode: boolean
  onExitSelectionMode: () => void
}

export default function SelectionToolbar({ totalCount, allIds, isSelectionMode, onExitSelectionMode }: SelectionToolbarProps) {
  const { selectedIds, clearSelection, selectAll } = useSelectionStore()
  const [showTagInput, setShowTagInput] = useState(false)
  const [tagInput, setTagInput] = useState('')
  const queryClient = useQueryClient()

  const selectedCount = selectedIds.size
  const hasSelection = selectedCount > 0
  const allSelected = selectedCount === totalCount && totalCount > 0

  const updateMutation = useMutation({
    mutationFn: batchApi.update,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['images'] })
      toast.success(`Updated ${selectedCount} images`)
    },
    onError: () => {
      toast.error('Failed to update images')
    },
  })

  const deleteMutation = useMutation({
    mutationFn: batchApi.delete,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['images'] })
      clearSelection()
      toast.success(`Moved ${selectedCount} images to trash`)
    },
    onError: () => {
      toast.error('Failed to delete images')
    },
  })

  const handleSetRating = (rating: number) => {
    updateMutation.mutate({
      ids: Array.from(selectedIds),
      rating,
    })
  }

  const handleToggleFavorite = (favorite: boolean) => {
    updateMutation.mutate({
      ids: Array.from(selectedIds),
      is_favorite: favorite,
    })
  }

  const handleAddTag = () => {
    const tag = tagInput.trim().toLowerCase()
    if (!tag) return

    updateMutation.mutate({
      ids: Array.from(selectedIds),
      add_tags: [tag],
    })
    setTagInput('')
    setShowTagInput(false)
  }

  const handleDelete = () => {
    if (confirm(`Move ${selectedCount} images to trash?`)) {
      deleteMutation.mutate({
        ids: Array.from(selectedIds),
      })
    }
  }

  const handleExport = (format: 'json' | 'csv' | 'prompts') => {
    const ids = Array.from(selectedIds)
    const idsParam = ids.map((id) => `ids=${id}`).join('&')

    let url: string
    if (format === 'prompts') {
      url = `/api/export/prompts?${idsParam}`
    } else {
      url = `/api/export/metadata?format=${format}&${idsParam}`
    }

    window.open(url, '_blank')
  }

  const handleSelectAll = () => {
    if (allSelected) {
      clearSelection()
    } else {
      selectAll(allIds)
    }
  }

  if (!isSelectionMode) return null

  const disabledClass = !hasSelection ? 'opacity-40 cursor-not-allowed' : ''

  return (
    <div className="fixed bottom-0 left-0 right-0 z-50 px-3 pb-[max(12px,env(safe-area-inset-bottom))] sm:left-1/2 sm:right-auto sm:-translate-x-1/2 sm:px-0 sm:pb-6">
      <div className="flex items-center gap-2 sm:gap-3 px-3 sm:px-4 py-2.5 sm:py-3 bg-gray-800 border border-gray-700 rounded-xl shadow-xl overflow-x-auto scrollbar-hide">
        <button
          onClick={handleSelectAll}
          className="flex items-center gap-1.5 sm:gap-2 px-2 sm:px-3 py-1.5 text-sm text-gray-300 hover:text-white transition-colors whitespace-nowrap shrink-0"
          title={allSelected ? 'Deselect all' : 'Select all'}
        >
          {allSelected ? <CheckSquare size={18} /> : <Square size={18} />}
          <span>{selectedCount}</span>
        </button>

        <div className="w-px h-6 bg-gray-700 shrink-0" />

        {/* Rating */}
        <div className={`flex items-center gap-0.5 sm:gap-1 shrink-0 ${disabledClass}`}>
          {[1, 2, 3, 4, 5].map((rating) => (
            <button
              key={rating}
              onClick={() => hasSelection && handleSetRating(rating)}
              disabled={!hasSelection}
              className="p-1 text-gray-500 hover:text-yellow-400 transition-colors disabled:hover:text-gray-500"
              title={`★${rating}: ${RATING_LABELS[rating]}`}
            >
              <Star size={18} />
            </button>
          ))}
        </div>

        <div className="w-px h-6 bg-gray-700 shrink-0" />

        {/* Favorite */}
        <button
          onClick={() => hasSelection && handleToggleFavorite(true)}
          disabled={!hasSelection}
          className={`p-1.5 sm:p-2 text-gray-500 hover:text-red-500 transition-colors shrink-0 disabled:hover:text-gray-500 ${disabledClass}`}
          title="Add to favorites"
        >
          <Heart size={18} />
        </button>

        {/* Tags */}
        {showTagInput ? (
          <div className="flex items-center gap-1.5 sm:gap-2 shrink-0">
            <input
              type="text"
              value={tagInput}
              onChange={(e) => setTagInput(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && handleAddTag()}
              placeholder="Tag"
              className="w-20 sm:w-24 px-2 py-1 bg-gray-700 border border-gray-600 rounded text-sm text-white placeholder-gray-400 focus:outline-none focus:ring-1 focus:ring-blue-500"
              autoFocus
            />
            <button
              onClick={handleAddTag}
              disabled={!hasSelection}
              className="px-2 py-1 bg-blue-600 text-white text-sm rounded hover:bg-blue-700 disabled:opacity-40 disabled:cursor-not-allowed"
            >
              Add
            </button>
            <button
              onClick={() => setShowTagInput(false)}
              className="p-1 text-gray-400 hover:text-white"
            >
              <X size={16} />
            </button>
          </div>
        ) : (
          <button
            onClick={() => setShowTagInput(true)}
            className={`p-1.5 sm:p-2 text-gray-500 hover:text-blue-400 transition-colors shrink-0 ${disabledClass}`}
            disabled={!hasSelection}
          >
            <Tag size={18} />
          </button>
        )}

        <div className="w-px h-6 bg-gray-700 shrink-0" />

        {/* Delete */}
        <button
          onClick={() => hasSelection && handleDelete()}
          disabled={!hasSelection}
          className={`p-1.5 sm:p-2 text-gray-500 hover:text-red-500 transition-colors shrink-0 disabled:hover:text-gray-500 ${disabledClass}`}
          title="Move to trash"
        >
          <Trash2 size={18} />
        </button>

        <div className="w-px h-6 bg-gray-700 shrink-0" />

        {/* Export */}
        <div className={`relative group shrink-0 ${disabledClass}`}>
          <button
            className="p-1.5 sm:p-2 text-gray-500 hover:text-green-400 transition-colors disabled:hover:text-gray-500"
            title="Export"
            disabled={!hasSelection}
          >
            <Download size={18} />
          </button>
          {hasSelection && (
            <div className="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 hidden group-hover:block">
              <div className="bg-gray-700 rounded-lg shadow-lg py-1 min-w-[100px]">
                <button
                  onClick={() => handleExport('json')}
                  className="w-full px-3 py-1.5 text-sm text-gray-300 hover:bg-gray-600 text-left"
                >
                  JSON
                </button>
                <button
                  onClick={() => handleExport('csv')}
                  className="w-full px-3 py-1.5 text-sm text-gray-300 hover:bg-gray-600 text-left"
                >
                  CSV
                </button>
                <button
                  onClick={() => handleExport('prompts')}
                  className="w-full px-3 py-1.5 text-sm text-gray-300 hover:bg-gray-600 text-left"
                >
                  Prompts
                </button>
              </div>
            </div>
          )}
        </div>

        <div className="w-px h-6 bg-gray-700 shrink-0" />

        {/* Exit selection mode */}
        <button
          onClick={onExitSelectionMode}
          className="p-1.5 sm:p-2 text-gray-500 hover:text-white transition-colors shrink-0"
          title="Exit selection mode"
        >
          <X size={18} />
        </button>
      </div>
    </div>
  )
}
