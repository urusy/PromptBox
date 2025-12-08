import { useState } from 'react'
import { Search, X, ChevronDown, ChevronUp } from 'lucide-react'
import type { ImageSearchParams } from '@/types/image'

interface SearchFormProps {
  params: ImageSearchParams
  onSearch: (params: ImageSearchParams) => void
}

const SOURCE_TOOLS = ['comfyui', 'a1111', 'forge', 'novelai']
const MODEL_TYPES = ['sd15', 'sdxl', 'pony', 'illustrious', 'flux', 'qwen']
const SORT_OPTIONS = [
  { value: 'created_at', label: 'Date Created' },
  { value: 'rating', label: 'Rating' },
  { value: 'model_name', label: 'Model Name' },
]
const GRID_FILTER_OPTIONS = [
  { value: '', label: 'All' },
  { value: 'grid', label: 'Grid Only' },
  { value: 'non-grid', label: 'Non-Grid Only' },
]

export default function SearchForm({ params, onSearch }: SearchFormProps) {
  const [isExpanded, setIsExpanded] = useState(false)
  const [localParams, setLocalParams] = useState<ImageSearchParams>(params)

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
    localParams.is_favorite ||
    localParams.is_xyz_grid !== undefined
  )

  return (
    <form onSubmit={handleSubmit} className="bg-gray-800 rounded-lg p-4 mb-6">
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
            <label className="block text-sm text-gray-400 mb-1">Min Rating</label>
            <select
              value={localParams.min_rating ?? ''}
              onChange={(e) => updateParam('min_rating', e.target.value ? parseInt(e.target.value) : undefined)}
              className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              <option value="">Any</option>
              {[1, 2, 3, 4, 5].map((rating) => (
                <option key={rating} value={rating}>
                  {rating}+ Stars
                </option>
              ))}
            </select>
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
    </form>
  )
}
