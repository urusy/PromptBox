import { useState, useEffect } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { Search, X, ChevronDown, ChevronUp, Save, Trash2, Bookmark } from 'lucide-react'
import toast from 'react-hot-toast'
import type { ImageSearchParams } from '@/types/image'
import type { SearchFilters } from '@/types/searchPreset'
import { searchPresetsApi } from '@/api/searchPresets'

interface SearchFormProps {
  params: ImageSearchParams
  onSearch: (params: ImageSearchParams) => void
}

const SOURCE_TOOLS = ['comfyui', 'a1111', 'forge', 'novelai']
const MODEL_TYPES = ['sd15', 'sdxl', 'pony', 'illustrious', 'flux', 'qwen']
const SORT_OPTIONS = [
  { value: 'created_at', label: 'Date Created' },
  { value: 'updated_at', label: 'Date Updated' },
  { value: 'rating', label: 'Rating' },
  { value: 'model_name', label: 'Model Name' },
]
const GRID_FILTER_OPTIONS = [
  { value: '', label: 'All' },
  { value: 'grid', label: 'Grid Only' },
  { value: 'non-grid', label: 'Non-Grid Only' },
]
const UPSCALE_FILTER_OPTIONS = [
  { value: '', label: 'All' },
  { value: 'upscaled', label: 'Upscaled Only' },
  { value: 'non-upscaled', label: 'Non-Upscaled Only' },
]
const RATING_MATCH_OPTIONS = [
  { value: 'min', label: '以上' },
  { value: 'exact', label: '同等' },
]

// Helper function to convert ImageSearchParams to SearchFilters (excluding page, per_page)
const paramsToFilters = (params: ImageSearchParams): SearchFilters => {
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const { page, per_page, ...filters } = params
  return filters
}

// Helper function to convert SearchFilters to ImageSearchParams
const filtersToParams = (filters: SearchFilters, currentParams: ImageSearchParams): ImageSearchParams => {
  return {
    ...filters,
    page: 1,
    per_page: currentParams.per_page || 24,
  }
}

export default function SearchForm({ params, onSearch }: SearchFormProps) {
  const [isExpanded, setIsExpanded] = useState(false)
  const [localParams, setLocalParams] = useState<ImageSearchParams>(params)
  const [showSaveModal, setShowSaveModal] = useState(false)
  const [presetName, setPresetName] = useState('')
  const [showPresetDropdown, setShowPresetDropdown] = useState(false)

  const queryClient = useQueryClient()

  // Sync localParams when params prop changes
  useEffect(() => {
    setLocalParams(params)
  }, [params])

  // Fetch presets
  const { data: presets = [] } = useQuery({
    queryKey: ['search-presets'],
    queryFn: searchPresetsApi.list,
  })

  // Create preset mutation
  const createPresetMutation = useMutation({
    mutationFn: searchPresetsApi.create,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['search-presets'] })
      toast.success('プリセットを保存しました')
      setShowSaveModal(false)
      setPresetName('')
    },
    onError: () => {
      toast.error('プリセットの保存に失敗しました')
    },
  })

  // Delete preset mutation
  const deletePresetMutation = useMutation({
    mutationFn: searchPresetsApi.delete,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['search-presets'] })
      toast.success('プリセットを削除しました')
    },
    onError: () => {
      toast.error('プリセットの削除に失敗しました')
    },
  })

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    onSearch({ ...localParams, page: 1 })
  }

  const handleReset = () => {
    const resetParams: ImageSearchParams = {
      page: 1,
      per_page: 24,
      sort_by: 'created_at',
      sort_order: 'desc',
    }
    setLocalParams(resetParams)
    onSearch(resetParams)
  }

  const updateParam = <K extends keyof ImageSearchParams>(
    key: K,
    value: ImageSearchParams[K]
  ) => {
    setLocalParams({ ...localParams, [key]: value })
  }

  const hasActiveFilters = !!(
    localParams.q ||
    localParams.source_tool ||
    localParams.model_type ||
    localParams.model_name ||
    localParams.min_rating ||
    localParams.exact_rating !== undefined ||
    localParams.is_favorite ||
    localParams.is_xyz_grid !== undefined ||
    localParams.is_upscaled !== undefined ||
    localParams.min_width ||
    localParams.min_height
  )

  const handleSavePreset = () => {
    if (!presetName.trim()) {
      toast.error('プリセット名を入力してください')
      return
    }
    createPresetMutation.mutate({
      name: presetName.trim(),
      filters: paramsToFilters(localParams),
    })
  }

  const handleSelectPreset = (presetId: string) => {
    setShowPresetDropdown(false)
    const preset = presets.find((p) => p.id === presetId)
    if (preset) {
      const newParams = filtersToParams(preset.filters, localParams)
      setLocalParams(newParams)
      onSearch(newParams)
    }
  }

  const handleDeletePreset = (e: React.MouseEvent, presetId: string) => {
    e.stopPropagation()
    if (confirm('このプリセットを削除しますか？')) {
      deletePresetMutation.mutate(presetId)
    }
  }

  return (
    <form onSubmit={handleSubmit} className="bg-gray-800 rounded-lg p-4 mb-6">
      {/* Preset selector row */}
      <div className="flex items-center gap-2 mb-3">
        <div className="relative">
          <button
            type="button"
            onClick={() => setShowPresetDropdown(!showPresetDropdown)}
            className="flex items-center gap-2 px-3 py-1.5 bg-gray-700 rounded-lg hover:bg-gray-600 transition-colors text-sm"
          >
            <Bookmark size={16} />
            <span>プリセット</span>
            <ChevronDown size={14} className={`transition-transform ${showPresetDropdown ? 'rotate-180' : ''}`} />
          </button>

          {showPresetDropdown && (
            <div className="absolute top-full left-0 mt-1 w-64 bg-gray-700 border border-gray-600 rounded-lg shadow-xl z-20">
              {presets.length === 0 ? (
                <div className="px-3 py-2 text-sm text-gray-400">
                  保存されたプリセットはありません
                </div>
              ) : (
                <div className="max-h-60 overflow-y-auto">
                  {presets.map((preset) => (
                    <div
                      key={preset.id}
                      className="flex items-center justify-between px-3 py-2 hover:bg-gray-600 cursor-pointer group"
                      onClick={() => handleSelectPreset(preset.id)}
                    >
                      <span className="text-sm text-white truncate flex-1">{preset.name}</span>
                      <button
                        type="button"
                        onClick={(e) => handleDeletePreset(e, preset.id)}
                        className="p-1 text-gray-400 hover:text-red-500 opacity-0 group-hover:opacity-100 transition-opacity"
                        title="削除"
                      >
                        <Trash2 size={14} />
                      </button>
                    </div>
                  ))}
                </div>
              )}
            </div>
          )}
        </div>

        {hasActiveFilters && (
          <button
            type="button"
            onClick={() => setShowSaveModal(true)}
            className="flex items-center gap-1.5 px-3 py-1.5 bg-gray-700 rounded-lg hover:bg-gray-600 transition-colors text-sm text-gray-300 hover:text-white"
            title="現在の条件を保存"
          >
            <Save size={14} />
            <span className="hidden sm:inline">保存</span>
          </button>
        )}
      </div>

      <div className="flex items-center gap-4">
        <div className="flex-1 relative">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400" size={18} />
          <input
            type="text"
            placeholder="Search prompts..."
            value={localParams.q || ''}
            onChange={(e) => updateParam('q', e.target.value || undefined)}
            className="w-full pl-10 pr-4 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>

        {hasActiveFilters && (
          <button
            type="button"
            onClick={handleReset}
            className="p-2 text-gray-400 hover:text-white transition-colors"
            title="Clear filters"
          >
            <X size={18} />
          </button>
        )}

        <button
          type="button"
          onClick={() => setIsExpanded(!isExpanded)}
          className="flex items-center gap-2 px-4 py-2 bg-gray-700 rounded-lg hover:bg-gray-600 transition-colors"
        >
          Filters
          {isExpanded ? <ChevronUp size={18} /> : <ChevronDown size={18} />}
          {hasActiveFilters && (
            <span className="w-2 h-2 bg-blue-500 rounded-full" />
          )}
        </button>

        <button
          type="submit"
          className="px-4 py-2 bg-blue-600 rounded-lg hover:bg-blue-700 transition-colors"
        >
          Search
        </button>
      </div>

      {isExpanded && (
        <div className="mt-4 pt-4 border-t border-gray-700 grid grid-cols-2 md:grid-cols-4 gap-4">
          <div>
            <label className="block text-sm text-gray-400 mb-1">Source Tool</label>
            <select
              value={localParams.source_tool || ''}
              onChange={(e) => updateParam('source_tool', e.target.value || undefined)}
              className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              <option value="">All</option>
              {SOURCE_TOOLS.map((tool) => (
                <option key={tool} value={tool}>
                  {tool.toUpperCase()}
                </option>
              ))}
            </select>
          </div>

          <div>
            <label className="block text-sm text-gray-400 mb-1">Model Type</label>
            <select
              value={localParams.model_type || ''}
              onChange={(e) => updateParam('model_type', e.target.value || undefined)}
              className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              <option value="">All</option>
              {MODEL_TYPES.map((type) => (
                <option key={type} value={type}>
                  {type.toUpperCase()}
                </option>
              ))}
            </select>
          </div>

          <div>
            <label className="block text-sm text-gray-400 mb-1">Model Name</label>
            <input
              type="text"
              placeholder="Filter by model..."
              value={localParams.model_name || ''}
              onChange={(e) => updateParam('model_name', e.target.value || undefined)}
              className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>

          <div>
            <label className="block text-sm text-gray-400 mb-1">Rating</label>
            <div className="flex gap-2">
              <select
                value={localParams.exact_rating ?? localParams.min_rating ?? ''}
                onChange={(e) => {
                  const val = e.target.value ? parseInt(e.target.value) : undefined
                  const isExactMode = localParams.exact_rating !== undefined
                  if (isExactMode) {
                    setLocalParams({ ...localParams, exact_rating: val, min_rating: undefined })
                  } else {
                    setLocalParams({ ...localParams, min_rating: val, exact_rating: undefined })
                  }
                }}
                className="flex-1 px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
              >
                <option value="">Any</option>
                <option value="0">★0</option>
                {[1, 2, 3, 4, 5].map((rating) => (
                  <option key={rating} value={rating}>
                    ★{rating}
                  </option>
                ))}
              </select>
              <select
                value={localParams.exact_rating !== undefined ? 'exact' : 'min'}
                onChange={(e) => {
                  const currentVal = localParams.exact_rating ?? localParams.min_rating
                  if (e.target.value === 'exact') {
                    setLocalParams({ ...localParams, exact_rating: currentVal, min_rating: undefined })
                  } else {
                    setLocalParams({ ...localParams, min_rating: currentVal, exact_rating: undefined })
                  }
                }}
                className="w-20 px-2 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
              >
                {RATING_MATCH_OPTIONS.map((option) => (
                  <option key={option.value} value={option.value}>
                    {option.label}
                  </option>
                ))}
              </select>
            </div>
          </div>

          <div>
            <label className="block text-sm text-gray-400 mb-1">Grid Filter</label>
            <select
              value={localParams.is_xyz_grid === true ? 'grid' : localParams.is_xyz_grid === false ? 'non-grid' : ''}
              onChange={(e) => {
                const val = e.target.value
                updateParam('is_xyz_grid', val === 'grid' ? true : val === 'non-grid' ? false : undefined)
              }}
              className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              {GRID_FILTER_OPTIONS.map((option) => (
                <option key={option.value} value={option.value}>
                  {option.label}
                </option>
              ))}
            </select>
          </div>

          <div>
            <label className="block text-sm text-gray-400 mb-1">Upscale</label>
            <select
              value={localParams.is_upscaled === true ? 'upscaled' : localParams.is_upscaled === false ? 'non-upscaled' : ''}
              onChange={(e) => {
                const val = e.target.value
                updateParam('is_upscaled', val === 'upscaled' ? true : val === 'non-upscaled' ? false : undefined)
              }}
              className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              {UPSCALE_FILTER_OPTIONS.map((option) => (
                <option key={option.value} value={option.value}>
                  {option.label}
                </option>
              ))}
            </select>
          </div>

          <div>
            <label className="block text-sm text-gray-400 mb-1">
              Min Width: {localParams.min_width ? `${localParams.min_width}px` : 'Any'}
            </label>
            <input
              type="range"
              min="0"
              max="4000"
              step="100"
              value={localParams.min_width || 0}
              onChange={(e) => updateParam('min_width', parseInt(e.target.value) || undefined)}
              className="w-full h-2 bg-gray-700 rounded-lg appearance-none cursor-pointer accent-blue-500"
            />
          </div>

          <div>
            <label className="block text-sm text-gray-400 mb-1">
              Min Height: {localParams.min_height ? `${localParams.min_height}px` : 'Any'}
            </label>
            <input
              type="range"
              min="0"
              max="4000"
              step="100"
              value={localParams.min_height || 0}
              onChange={(e) => updateParam('min_height', parseInt(e.target.value) || undefined)}
              className="w-full h-2 bg-gray-700 rounded-lg appearance-none cursor-pointer accent-blue-500"
            />
          </div>

          <div>
            <label className="block text-sm text-gray-400 mb-1">Sort By</label>
            <select
              value={localParams.sort_by || 'created_at'}
              onChange={(e) => updateParam('sort_by', e.target.value)}
              className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              {SORT_OPTIONS.map((option) => (
                <option key={option.value} value={option.value}>
                  {option.label}
                </option>
              ))}
            </select>
          </div>

          <div>
            <label className="block text-sm text-gray-400 mb-1">Order</label>
            <select
              value={localParams.sort_order || 'desc'}
              onChange={(e) => updateParam('sort_order', e.target.value as 'asc' | 'desc')}
              className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              <option value="desc">Descending</option>
              <option value="asc">Ascending</option>
            </select>
          </div>

          <div className="flex items-end">
            <label className="flex items-center gap-2 cursor-pointer">
              <input
                type="checkbox"
                checked={localParams.is_favorite === true}
                onChange={(e) => updateParam('is_favorite', e.target.checked ? true : undefined)}
                className="w-4 h-4 rounded border-gray-600 bg-gray-700 text-blue-600 focus:ring-blue-500"
              />
              <span className="text-sm text-gray-300">Favorites only</span>
            </label>
          </div>
        </div>
      )}

      {/* Save preset modal */}
      {showSaveModal && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50" onClick={() => setShowSaveModal(false)}>
          <div className="bg-gray-800 rounded-lg p-6 w-full max-w-sm mx-4" onClick={(e) => e.stopPropagation()}>
            <h3 className="text-lg font-semibold text-white mb-4">検索条件を保存</h3>
            <input
              type="text"
              placeholder="プリセット名"
              value={presetName}
              onChange={(e) => setPresetName(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && handleSavePreset()}
              className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 mb-4"
              autoFocus
            />
            <div className="flex gap-2 justify-end">
              <button
                type="button"
                onClick={() => setShowSaveModal(false)}
                className="px-4 py-2 text-gray-400 hover:text-white transition-colors"
              >
                キャンセル
              </button>
              <button
                type="button"
                onClick={handleSavePreset}
                disabled={createPresetMutation.isPending}
                className="px-4 py-2 bg-blue-600 rounded-lg hover:bg-blue-700 transition-colors disabled:opacity-50"
              >
                保存
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Click outside to close dropdown */}
      {showPresetDropdown && (
        <div className="fixed inset-0 z-10" onClick={() => setShowPresetDropdown(false)} />
      )}
    </form>
  )
}
