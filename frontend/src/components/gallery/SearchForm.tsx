import { useState, useEffect, useRef, useMemo } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { Search, X, ChevronDown, ChevronUp, Save, Trash2, Bookmark, Tag } from 'lucide-react'
import toast from 'react-hot-toast'
import type { ImageSearchParams } from '@/types/image'
import type { SearchFilters } from '@/types/searchPreset'
import { searchPresetsApi } from '@/api/searchPresets'
import { statsApi } from '@/api/stats'
import { tagsApi } from '@/api/tags'

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
const ORIENTATION_OPTIONS = [
  { value: '', label: 'All' },
  { value: 'portrait', label: 'Portrait' },
  { value: 'landscape', label: 'Landscape' },
  { value: 'square', label: 'Square' },
]
const RATING_MATCH_OPTIONS = [
  { value: 'min', label: '以上' },
  { value: 'exact', label: '同等' },
]
const DATE_RANGE_OPTIONS = [
  { value: '', label: 'All' },
  { value: 'today', label: 'Today' },
  { value: '7days', label: 'Past 7 Days' },
  { value: '30days', label: 'Past 30 Days' },
  { value: 'month', label: 'This Month' },
]

// Helper to convert date range option to ISO date string
const getDateFromValue = (option: string): string | undefined => {
  if (!option) return undefined
  const now = new Date()
  switch (option) {
    case 'today':
      return new Date(now.getFullYear(), now.getMonth(), now.getDate()).toISOString()
    case '7days':
      return new Date(now.getTime() - 7 * 24 * 60 * 60 * 1000).toISOString()
    case '30days':
      return new Date(now.getTime() - 30 * 24 * 60 * 60 * 1000).toISOString()
    case 'month':
      return new Date(now.getFullYear(), now.getMonth(), 1).toISOString()
    default:
      return undefined
  }
}

// Helper to convert ISO date string back to option value (for display)
const getDateRangeOption = (dateFrom: string | undefined): string => {
  if (!dateFrom) return ''
  const date = new Date(dateFrom)
  const now = new Date()
  const today = new Date(now.getFullYear(), now.getMonth(), now.getDate())
  const thisMonth = new Date(now.getFullYear(), now.getMonth(), 1)

  if (date.getTime() === today.getTime()) return 'today'
  if (date.getTime() === thisMonth.getTime()) return 'month'

  const diffDays = Math.round((now.getTime() - date.getTime()) / (24 * 60 * 60 * 1000))
  if (diffDays >= 6 && diffDays <= 8) return '7days'
  if (diffDays >= 29 && diffDays <= 31) return '30days'

  return ''
}

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

// Helper function to check if filters have any meaningful values
const filtersHaveActiveConditions = (filters: SearchFilters): boolean => {
  const { sort_by: _sort_by, sort_order: _sort_order, ...rest } = filters
  // Ignore sort_by and sort_order for determining if filters are "active"
  return Object.values(rest).some((val) => {
    if (val === undefined || val === null || val === '' || val === 0) return false
    if (Array.isArray(val) && val.length === 0) return false
    return true
  })
}

// Helper function to compare filters (ignoring undefined vs not-present)
const filtersMatch = (a: SearchFilters, b: SearchFilters): boolean => {
  const keys: (keyof SearchFilters)[] = [
    'q', 'source_tool', 'model_type', 'model_name', 'sampler_name',
    'min_rating', 'exact_rating', 'is_favorite', 'needs_improvement',
    'tags', 'lora_name', 'is_xyz_grid', 'is_upscaled', 'orientation',
    'min_width', 'min_height', 'date_from', 'sort_by', 'sort_order'
  ]

  for (const key of keys) {
    const aVal = a[key]
    const bVal = b[key]

    // Treat undefined, null, empty string, and 0 as equivalent for comparison
    const aEmpty = aVal === undefined || aVal === null || aVal === '' || aVal === 0
    const bEmpty = bVal === undefined || bVal === null || bVal === '' || bVal === 0

    if (aEmpty && bEmpty) continue
    if (aEmpty !== bEmpty) return false

    // For arrays, compare contents
    if (Array.isArray(aVal) && Array.isArray(bVal)) {
      if (aVal.length !== bVal.length) return false
      if (!aVal.every((v, i) => v === bVal[i])) return false
      continue
    }

    if (aVal !== bVal) return false
  }

  return true
}

export default function SearchForm({ params, onSearch }: SearchFormProps) {
  const [isExpanded, setIsExpanded] = useState(false)
  const [localParams, setLocalParams] = useState<ImageSearchParams>(params)
  const [showSaveModal, setShowSaveModal] = useState(false)
  const [presetName, setPresetName] = useState('')
  const [showPresetDropdown, setShowPresetDropdown] = useState(false)
  const [selectedPresetId, setSelectedPresetId] = useState<string | null>(null)
  const [showModelDropdown, setShowModelDropdown] = useState(false)
  const [showLoraDropdown, setShowLoraDropdown] = useState(false)
  const [showTagDropdown, setShowTagDropdown] = useState(false)
  const [tagSearchQuery, setTagSearchQuery] = useState('')
  const modelInputRef = useRef<HTMLInputElement>(null)
  const loraInputRef = useRef<HTMLInputElement>(null)
  const tagInputRef = useRef<HTMLInputElement>(null)
  const saveModalRef = useRef<HTMLDivElement>(null)

  const queryClient = useQueryClient()

  // Fetch presets
  const { data: presets = [] } = useQuery({
    queryKey: ['search-presets'],
    queryFn: searchPresetsApi.list,
  })

  // Fetch model names for dropdown
  const { data: modelList } = useQuery({
    queryKey: ['models-for-filter'],
    queryFn: () => statsApi.getModelsForAnalysis(1),  // min_count=1 to get all models
  })

  // Fetch LoRA names for dropdown
  const { data: loraList } = useQuery({
    queryKey: ['loras-for-filter'],
    queryFn: () => statsApi.getLorasForFilter(1),  // min_count=1 to get all LoRAs
  })

  // Fetch Sampler names for dropdown
  const { data: samplerList } = useQuery({
    queryKey: ['samplers-for-filter'],
    queryFn: () => statsApi.getSamplersForFilter(1),  // min_count=1 to get all Samplers
  })

  // Fetch tags for dropdown
  const { data: tagList = [] } = useQuery({
    queryKey: ['tags-for-filter', tagSearchQuery],
    queryFn: () => tagsApi.list(tagSearchQuery || undefined, 50),
  })

  // Extract model name (after last backslash), dedupe, sort alphabetically, and filter
  const processedModels = useMemo(() => {
    if (!modelList?.models) return []

    // Extract name after last backslash and dedupe
    const modelNames = new Set<string>()
    modelList.models.forEach((model) => {
      const lastBackslash = model.lastIndexOf('\\')
      const name = lastBackslash >= 0 ? model.slice(lastBackslash + 1) : model
      if (name) modelNames.add(name)
    })

    // Convert to array and sort alphabetically (case-insensitive)
    return Array.from(modelNames).sort((a, b) =>
      a.toLowerCase().localeCompare(b.toLowerCase())
    )
  }, [modelList?.models])

  // Filter models based on current input
  const filteredModels = processedModels.filter(
    (model) => !localParams.model_name || model.toLowerCase().includes(localParams.model_name.toLowerCase())
  )

  // Process and filter LoRA names
  const processedLoras = useMemo(() => {
    if (!loraList?.loras) return []
    // Sort alphabetically (case-insensitive)
    return [...loraList.loras].sort((a, b) =>
      a.toLowerCase().localeCompare(b.toLowerCase())
    )
  }, [loraList?.loras])

  // Filter LoRAs based on current input
  const filteredLoras = processedLoras.filter(
    (lora) => !localParams.lora_name || lora.toLowerCase().includes(localParams.lora_name.toLowerCase())
  )

  // Filter tags - exclude already selected tags
  const filteredTags = useMemo(() => {
    const selectedTags = localParams.tags || []
    return tagList.filter((tag) => !selectedTags.includes(tag))
  }, [tagList, localParams.tags])

  // Sync localParams when params prop changes
  useEffect(() => {
    setLocalParams(params)
  }, [params])

  // Check if current params match any preset and update selectedPresetId
  useEffect(() => {
    if (presets.length === 0) return

    const currentFilters = paramsToFilters(params)

    // Don't match presets if no filters are active (initial/reset state)
    if (!filtersHaveActiveConditions(currentFilters)) {
      setSelectedPresetId(null)
      return
    }

    const matchingPreset = presets.find((preset) =>
      filtersMatch(currentFilters, preset.filters)
    )

    setSelectedPresetId(matchingPreset?.id ?? null)
  }, [params, presets])

  // Focus trap for save modal
  useEffect(() => {
    if (!showSaveModal || !saveModalRef.current) return

    const modal = saveModalRef.current
    const focusableElements = modal.querySelectorAll<HTMLElement>(
      'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
    )
    const firstFocusable = focusableElements[0]
    const lastFocusable = focusableElements[focusableElements.length - 1]

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        setShowSaveModal(false)
        return
      }

      if (e.key !== 'Tab') return

      if (e.shiftKey) {
        if (document.activeElement === firstFocusable) {
          e.preventDefault()
          lastFocusable?.focus()
        }
      } else {
        if (document.activeElement === lastFocusable) {
          e.preventDefault()
          firstFocusable?.focus()
        }
      }
    }

    modal.addEventListener('keydown', handleKeyDown)
    return () => modal.removeEventListener('keydown', handleKeyDown)
  }, [showSaveModal])

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

  const addTag = (tag: string) => {
    const currentTags = localParams.tags || []
    if (!currentTags.includes(tag)) {
      setLocalParams({ ...localParams, tags: [...currentTags, tag] })
    }
    setTagSearchQuery('')
    setShowTagDropdown(false)
  }

  const removeTag = (tag: string) => {
    const currentTags = localParams.tags || []
    const newTags = currentTags.filter((t) => t !== tag)
    setLocalParams({ ...localParams, tags: newTags.length > 0 ? newTags : undefined })
  }

  const hasActiveFilters = !!(
    localParams.q ||
    localParams.source_tool ||
    localParams.model_type ||
    localParams.model_name ||
    localParams.sampler_name ||
    localParams.lora_name ||
    localParams.min_rating ||
    localParams.exact_rating !== undefined ||
    localParams.is_favorite ||
    localParams.is_xyz_grid !== undefined ||
    localParams.is_upscaled !== undefined ||
    localParams.orientation ||
    localParams.min_width ||
    localParams.min_height ||
    localParams.date_from ||
    (localParams.tags && localParams.tags.length > 0)
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

  // Get selected preset name
  const selectedPreset = presets.find((p) => p.id === selectedPresetId)

  return (
    <form onSubmit={handleSubmit} className="bg-gray-800 rounded-lg p-4 mb-6">
      {/* Preset selector row */}
      <div className="flex items-center gap-2 mb-3">
        <div className="relative">
          <button
            type="button"
            onClick={() => setShowPresetDropdown(!showPresetDropdown)}
            aria-label="プリセット選択"
            aria-expanded={showPresetDropdown}
            aria-haspopup="listbox"
            className={`flex items-center gap-2 px-3 py-1.5 rounded-lg transition-colors text-sm ${
              selectedPreset
                ? 'bg-blue-600 hover:bg-blue-700 text-white'
                : 'bg-gray-700 hover:bg-gray-600'
            }`}
          >
            <Bookmark size={16} className={selectedPreset ? 'fill-current' : ''} />
            <span className="max-w-32 truncate">{selectedPreset?.name || 'プリセット'}</span>
            <ChevronDown size={14} className={`transition-transform ${showPresetDropdown ? 'rotate-180' : ''}`} />
          </button>

          {showPresetDropdown && (
            <div
              role="listbox"
              aria-label="プリセット一覧"
              className="absolute top-full left-0 mt-1 w-64 bg-gray-700 border border-gray-600 rounded-lg shadow-xl z-20"
            >
              {presets.length === 0 ? (
                <div className="px-3 py-2 text-sm text-gray-400">
                  保存されたプリセットはありません
                </div>
              ) : (
                <div className="max-h-60 overflow-y-auto">
                  {presets.map((preset) => (
                    <div
                      key={preset.id}
                      className={`flex items-center justify-between px-3 py-2 cursor-pointer group ${
                        preset.id === selectedPresetId
                          ? 'bg-blue-600/30 hover:bg-blue-600/40'
                          : 'hover:bg-gray-600'
                      }`}
                      onClick={() => handleSelectPreset(preset.id)}
                    >
                      <span className={`text-sm truncate flex-1 ${
                        preset.id === selectedPresetId ? 'text-blue-300 font-medium' : 'text-white'
                      }`}>{preset.name}</span>
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
        <div className="mt-4 pt-4 border-t border-gray-700 space-y-4">
          {/* Row 1: Generation Source */}
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
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

            <div className="relative">
              <label className="block text-sm text-gray-400 mb-1">Model Name</label>
              <div className="relative">
                <input
                  ref={modelInputRef}
                  type="text"
                  placeholder="Filter by model..."
                  value={localParams.model_name || ''}
                  onChange={(e) => {
                    updateParam('model_name', e.target.value || undefined)
                    setShowModelDropdown(true)
                  }}
                  onFocus={() => setShowModelDropdown(true)}
                  onBlur={() => {
                    // Delay to allow click on dropdown items
                    setTimeout(() => setShowModelDropdown(false), 150)
                  }}
                  className={`w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 ${localParams.model_name ? 'pr-14' : 'pr-8'}`}
                />
                {localParams.model_name && (
                  <button
                    type="button"
                    onClick={() => {
                      updateParam('model_name', undefined)
                      modelInputRef.current?.focus()
                    }}
                    className="absolute right-8 top-1/2 -translate-y-1/2 text-gray-400 hover:text-white"
                    title="クリア"
                  >
                    <X size={16} />
                  </button>
                )}
                <button
                  type="button"
                  onClick={() => {
                    setShowModelDropdown(!showModelDropdown)
                    modelInputRef.current?.focus()
                  }}
                  className="absolute right-2 top-1/2 -translate-y-1/2 text-gray-400 hover:text-white"
                >
                  <ChevronDown size={16} className={`transition-transform ${showModelDropdown ? 'rotate-180' : ''}`} />
                </button>
              </div>
              {showModelDropdown && filteredModels.length > 0 && (
                <div className="absolute top-full left-0 right-0 mt-1 bg-gray-700 border border-gray-600 rounded-lg shadow-xl z-20 max-h-60 overflow-y-auto">
                  {filteredModels.slice(0, 20).map((model) => (
                    <button
                      key={model}
                      type="button"
                      onMouseDown={(e) => {
                        e.preventDefault()
                        updateParam('model_name', model)
                        setShowModelDropdown(false)
                      }}
                      className={`w-full text-left px-3 py-2 text-sm transition-colors ${
                        localParams.model_name === model
                          ? 'bg-blue-600 text-white'
                          : 'text-gray-300 hover:bg-gray-600'
                      }`}
                    >
                      {model}
                    </button>
                  ))}
                  {filteredModels.length > 20 && (
                    <div className="px-3 py-2 text-xs text-gray-500 border-t border-gray-600">
                      他 {filteredModels.length - 20} 件...
                    </div>
                  )}
                </div>
              )}
            </div>

            <div>
              <label className="block text-sm text-gray-400 mb-1">Sampler</label>
              <select
                value={localParams.sampler_name || ''}
                onChange={(e) => updateParam('sampler_name', e.target.value || undefined)}
                className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
              >
                <option value="">All</option>
                {samplerList?.samplers.map((sampler) => (
                  <option key={sampler} value={sampler}>
                    {sampler}
                  </option>
                ))}
              </select>
            </div>
          </div>

          {/* Row 2: LoRA & Image Attributes */}
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            <div className="relative">
              <label className="block text-sm text-gray-400 mb-1">LoRA</label>
              <div className="relative">
                <input
                  ref={loraInputRef}
                  type="text"
                  placeholder="Filter by LoRA..."
                  value={localParams.lora_name || ''}
                  onChange={(e) => {
                    updateParam('lora_name', e.target.value || undefined)
                    setShowLoraDropdown(true)
                  }}
                  onFocus={() => setShowLoraDropdown(true)}
                  onBlur={() => {
                    // Delay to allow click on dropdown items
                    setTimeout(() => setShowLoraDropdown(false), 150)
                  }}
                  className={`w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 ${localParams.lora_name ? 'pr-14' : 'pr-8'}`}
                />
                {localParams.lora_name && (
                  <button
                    type="button"
                    onClick={() => {
                      updateParam('lora_name', undefined)
                      loraInputRef.current?.focus()
                    }}
                    className="absolute right-8 top-1/2 -translate-y-1/2 text-gray-400 hover:text-white"
                    title="クリア"
                  >
                    <X size={16} />
                  </button>
                )}
                <button
                  type="button"
                  onClick={() => {
                    setShowLoraDropdown(!showLoraDropdown)
                    loraInputRef.current?.focus()
                  }}
                  className="absolute right-2 top-1/2 -translate-y-1/2 text-gray-400 hover:text-white"
                >
                  <ChevronDown size={16} className={`transition-transform ${showLoraDropdown ? 'rotate-180' : ''}`} />
                </button>
              </div>
              {showLoraDropdown && filteredLoras.length > 0 && (
                <div className="absolute top-full left-0 right-0 mt-1 bg-gray-700 border border-gray-600 rounded-lg shadow-xl z-20 max-h-60 overflow-y-auto">
                  {filteredLoras.slice(0, 20).map((lora) => (
                    <button
                      key={lora}
                      type="button"
                      onMouseDown={(e) => {
                        e.preventDefault()
                        updateParam('lora_name', lora)
                        setShowLoraDropdown(false)
                      }}
                      className={`w-full text-left px-3 py-2 text-sm transition-colors ${
                        localParams.lora_name === lora
                          ? 'bg-blue-600 text-white'
                          : 'text-gray-300 hover:bg-gray-600'
                      }`}
                    >
                      {lora}
                    </button>
                  ))}
                  {filteredLoras.length > 20 && (
                    <div className="px-3 py-2 text-xs text-gray-500 border-t border-gray-600">
                      他 {filteredLoras.length - 20} 件...
                    </div>
                  )}
                </div>
              )}
            </div>

            <div>
              <label className="block text-sm text-gray-400 mb-1">Orientation</label>
              <select
                value={localParams.orientation || ''}
                onChange={(e) => updateParam('orientation', e.target.value || undefined)}
                className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
              >
                {ORIENTATION_OPTIONS.map((option) => (
                  <option key={option.value} value={option.value}>
                    {option.label}
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
          </div>

          {/* Row 2.5: Tags */}
          <div>
            <label className="block text-sm text-gray-400 mb-1 flex items-center gap-1">
              <Tag size={14} />
              Tags (AND)
            </label>
            <div className="flex flex-wrap gap-2 mb-2">
              {(localParams.tags || []).map((tag) => (
                <span
                  key={tag}
                  className="inline-flex items-center gap-1 px-2 py-1 bg-blue-600/30 text-blue-300 rounded-lg text-sm"
                >
                  {tag}
                  <button
                    type="button"
                    onClick={() => removeTag(tag)}
                    className="hover:text-white"
                    title="Remove tag"
                  >
                    <X size={14} />
                  </button>
                </span>
              ))}
            </div>
            <div className="relative">
              <input
                ref={tagInputRef}
                type="text"
                placeholder="Add tag..."
                value={tagSearchQuery}
                onChange={(e) => {
                  setTagSearchQuery(e.target.value)
                  setShowTagDropdown(true)
                }}
                onFocus={() => setShowTagDropdown(true)}
                onBlur={() => {
                  setTimeout(() => setShowTagDropdown(false), 150)
                }}
                onKeyDown={(e) => {
                  if (e.key === 'Enter' && tagSearchQuery.trim()) {
                    e.preventDefault()
                    addTag(tagSearchQuery.trim())
                  }
                }}
                className="w-full md:w-64 px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
              {showTagDropdown && filteredTags.length > 0 && (
                <div className="absolute top-full left-0 w-full md:w-64 mt-1 bg-gray-700 border border-gray-600 rounded-lg shadow-xl z-20 max-h-48 overflow-y-auto">
                  {filteredTags.slice(0, 15).map((tag) => (
                    <button
                      key={tag}
                      type="button"
                      onMouseDown={(e) => {
                        e.preventDefault()
                        addTag(tag)
                      }}
                      className="w-full text-left px-3 py-2 text-sm text-gray-300 hover:bg-gray-600 transition-colors"
                    >
                      {tag}
                    </button>
                  ))}
                </div>
              )}
            </div>
          </div>

          {/* Row 3: Rating & Time */}
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            <div className="col-span-2">
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
                <button
                  type="button"
                  onClick={() => {
                    if (localParams.exact_rating === 0) {
                      setLocalParams({ ...localParams, exact_rating: undefined, min_rating: undefined })
                    } else {
                      setLocalParams({ ...localParams, exact_rating: 0, min_rating: undefined })
                    }
                  }}
                  className={`px-3 py-2 rounded-lg border transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 whitespace-nowrap ${
                    localParams.exact_rating === 0
                      ? 'bg-blue-600 border-blue-500 text-white'
                      : 'bg-gray-700 border-gray-600 text-gray-400 hover:text-white hover:border-gray-500'
                  }`}
                  title="未評価のみ表示"
                >
                  {localParams.exact_rating === 0 ? '未評価 ON' : '未評価'}
                </button>
              </div>
            </div>

            <div>
              <label className="block text-sm text-gray-400 mb-1">Favorites</label>
              <button
                type="button"
                onClick={() => updateParam('is_favorite', localParams.is_favorite ? undefined : true)}
                className={`w-full px-3 py-2 rounded-lg border transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 ${
                  localParams.is_favorite
                    ? 'bg-blue-600 border-blue-500 text-white'
                    : 'bg-gray-700 border-gray-600 text-gray-400 hover:text-white hover:border-gray-500'
                }`}
              >
                {localParams.is_favorite ? '★ Favorites ON' : '☆ Favorites'}
              </button>
            </div>

            <div>
              <label className="block text-sm text-gray-400 mb-1">Date Range</label>
              <select
                value={getDateRangeOption(localParams.date_from)}
                onChange={(e) => updateParam('date_from', getDateFromValue(e.target.value))}
                className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
              >
                {DATE_RANGE_OPTIONS.map((option) => (
                  <option key={option.value} value={option.value}>
                    {option.label}
                  </option>
                ))}
              </select>
            </div>
          </div>

          {/* Row 4: Dimensions & Sorting */}
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            <div className="col-span-2">
              <label className="block text-sm text-gray-400 mb-1">
                Min Size: {localParams.min_width || localParams.min_height
                  ? `${localParams.min_width || 0} × ${localParams.min_height || 0}px`
                  : 'Any'}
              </label>
              <div className="flex gap-3 items-center">
                <div className="flex-1">
                  <input
                    type="range"
                    min="0"
                    max="5000"
                    step="100"
                    value={localParams.min_width || 0}
                    onChange={(e) => updateParam('min_width', parseInt(e.target.value) || undefined)}
                    className="w-full h-2 bg-gray-700 rounded-lg appearance-none cursor-pointer accent-blue-500"
                    title={`Width: ${localParams.min_width || 0}px`}
                  />
                  <div className="text-xs text-gray-500 text-center mt-1">W</div>
                </div>
                <span className="text-gray-500">×</span>
                <div className="flex-1">
                  <input
                    type="range"
                    min="0"
                    max="5000"
                    step="100"
                    value={localParams.min_height || 0}
                    onChange={(e) => updateParam('min_height', parseInt(e.target.value) || undefined)}
                    className="w-full h-2 bg-gray-700 rounded-lg appearance-none cursor-pointer accent-blue-500"
                    title={`Height: ${localParams.min_height || 0}px`}
                  />
                  <div className="text-xs text-gray-500 text-center mt-1">H</div>
                </div>
              </div>
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
          </div>
        </div>
      )}

      {/* Save preset modal */}
      {showSaveModal && (
        <div
          className="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
          onClick={() => setShowSaveModal(false)}
          role="presentation"
        >
          <div
            ref={saveModalRef}
            role="dialog"
            aria-modal="true"
            aria-labelledby="save-preset-title"
            className="bg-gray-800 rounded-lg p-6 w-full max-w-sm mx-4"
            onClick={(e) => e.stopPropagation()}
          >
            <h3 id="save-preset-title" className="text-lg font-semibold text-white mb-4">検索条件を保存</h3>
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
