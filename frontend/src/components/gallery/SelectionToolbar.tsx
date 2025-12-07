import { useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { X, Star, Heart, Trash2, Tag, CheckSquare, Square, Download } from 'lucide-react'
import toast from 'react-hot-toast'
import { batchApi } from '@/api/batch'
import { useSelectionStore } from '@/stores/selectionStore'

interface SelectionToolbarProps {
  totalCount: number
  allIds: string[]
}

export default function SelectionToolbar({ totalCount, allIds }: SelectionToolbarProps) {
  const { selectedIds, clearSelection, selectAll } = useSelectionStore()
  const [showTagInput, setShowTagInput] = useState(false)
  const [tagInput, setTagInput] = useState('')
  const queryClient = useQueryClient()

  const selectedCount = selectedIds.size
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

  if (selectedCount === 0) return null

  return (
    <div className="fixed bottom-6 left-1/2 -translate-x-1/2 z-50">
      <div className="flex items-center gap-3 px-4 py-3 bg-gray-800 border border-gray-700 rounded-xl shadow-xl">
        <button
          onClick={handleSelectAll}
          className="flex items-center gap-2 px-3 py-1.5 text-sm text-gray-300 hover:text-white transition-colors"
          title={allSelected ? 'Deselect all' : 'Select all'}
        >
          {allSelected ? <CheckSquare size={18} /> : <Square size={18} />}
          <span>{selectedCount} selected</span>
        </button>

        <div className="w-px h-6 bg-gray-700" />

        {/* Rating */}
        <div className="flex items-center gap-1">
          {[1, 2, 3, 4, 5].map((rating) => (
            <button
              key={rating}
              onClick={() => handleSetRating(rating)}
              className="p-1 text-gray-500 hover:text-yellow-400 transition-colors"
              title={`Set rating to ${rating}`}
            >
              <Star size={18} />
            </button>
          ))}
        </div>

        <div className="w-px h-6 bg-gray-700" />

        {/* Favorite */}
        <button
          onClick={() => handleToggleFavorite(true)}
          className="p-2 text-gray-500 hover:text-red-500 transition-colors"
          title="Add to favorites"
        >
          <Heart size={18} />
        </button>

        {/* Tags */}
        {showTagInput ? (
          <div className="flex items-center gap-2">
            <input
              type="text"
              value={tagInput}
              onChange={(e) => setTagInput(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && handleAddTag()}
              placeholder="Tag name"
              className="w-24 px-2 py-1 bg-gray-700 border border-gray-600 rounded text-sm text-white placeholder-gray-400 focus:outline-none focus:ring-1 focus:ring-blue-500"
              autoFocus
            />
            <button
              onClick={handleAddTag}
              className="px-2 py-1 bg-blue-600 text-white text-sm rounded hover:bg-blue-700"
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
            className="p-2 text-gray-500 hover:text-blue-400 transition-colors"
            title="Add tag"
          >
            <Tag size={18} />
          </button>
        )}

        <div className="w-px h-6 bg-gray-700" />

        {/* Delete */}
        <button
          onClick={handleDelete}
          className="p-2 text-gray-500 hover:text-red-500 transition-colors"
          title="Move to trash"
        >
          <Trash2 size={18} />
        </button>

        <div className="w-px h-6 bg-gray-700" />

        {/* Export */}
        <div className="relative group">
          <button
            className="p-2 text-gray-500 hover:text-green-400 transition-colors"
            title="Export"
          >
            <Download size={18} />
          </button>
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
        </div>

        <div className="w-px h-6 bg-gray-700" />

        {/* Clear selection */}
        <button
          onClick={clearSelection}
          className="p-2 text-gray-500 hover:text-white transition-colors"
          title="Clear selection"
        >
          <X size={18} />
        </button>
      </div>
    </div>
  )
}
