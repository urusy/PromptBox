import { useState, useMemo } from 'react'
import { useQuery } from '@tanstack/react-query'
import { useNavigate, useSearchParams } from 'react-router-dom'
import { Search, Star, Image, SortAsc, SortDesc, Layers } from 'lucide-react'
import { lorasApi } from '@/api/models'
import type { LoraSearchParams } from '@/types/model'

export default function LorasPage() {
  const navigate = useNavigate()
  const [searchParams, setSearchParams] = useSearchParams()

  // Parse search params
  const params: LoraSearchParams = useMemo(
    () => ({
      q: searchParams.get('q') || undefined,
      min_count: searchParams.get('min_count') ? parseInt(searchParams.get('min_count')!) : 1,
      min_rating: searchParams.get('min_rating')
        ? parseFloat(searchParams.get('min_rating')!)
        : undefined,
      sort_by: (searchParams.get('sort_by') as 'count' | 'rating' | 'name') || 'count',
      sort_order: (searchParams.get('sort_order') as 'asc' | 'desc') || 'desc',
      limit: 50,
      offset: searchParams.get('offset') ? parseInt(searchParams.get('offset')!) : 0,
    }),
    [searchParams]
  )

  // Local state for form inputs
  const [searchQuery, setSearchQuery] = useState(params.q || '')

  const { data, isLoading, error } = useQuery({
    queryKey: ['loras', params],
    queryFn: () => lorasApi.list(params),
  })

  const updateParams = (updates: Partial<LoraSearchParams>) => {
    const newParams = new URLSearchParams(searchParams)

    Object.entries(updates).forEach(([key, value]) => {
      if (value === undefined || value === '' || value === null) {
        newParams.delete(key)
      } else {
        newParams.set(key, String(value))
      }
    })

    // Reset offset when filters change
    if (!('offset' in updates)) {
      newParams.delete('offset')
    }

    setSearchParams(newParams)
  }

  const handleSearch = (e: React.FormEvent) => {
    e.preventDefault()
    updateParams({ q: searchQuery || undefined })
  }

  const toggleSort = (field: 'count' | 'rating' | 'name') => {
    if (params.sort_by === field) {
      updateParams({ sort_order: params.sort_order === 'desc' ? 'asc' : 'desc' })
    } else {
      updateParams({ sort_by: field, sort_order: 'desc' })
    }
  }

  const SortIcon = params.sort_order === 'desc' ? SortDesc : SortAsc

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center gap-3">
        <Layers size={28} className="text-orange-400" />
        <h1 className="text-2xl font-bold">LoRAs</h1>
        {data && <span className="text-gray-400 text-sm">({data.total} LoRAs)</span>}
      </div>

      {/* Search & Filters */}
      <div className="bg-gray-800 rounded-lg p-4 space-y-4">
        {/* Search */}
        <form onSubmit={handleSearch} className="flex flex-col sm:flex-row gap-2">
          <div className="relative flex-1">
            <Search size={18} className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400" />
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="Search LoRAs..."
              className="w-full pl-10 pr-4 py-2 bg-gray-700 border border-gray-600 rounded-lg focus:outline-none focus:ring-2 focus:ring-orange-500"
            />
          </div>
          <button
            type="submit"
            className="px-4 py-2 bg-orange-600 hover:bg-orange-700 rounded-lg transition-colors"
          >
            Search
          </button>
        </form>

        {/* Filters */}
        <div className="flex flex-wrap gap-4">
          {/* Min Count */}
          <div className="flex items-center gap-2">
            <label className="text-sm text-gray-400">Min images:</label>
            <input
              type="number"
              value={params.min_count || 1}
              onChange={(e) => updateParams({ min_count: parseInt(e.target.value) || 1 })}
              min={1}
              className="w-20 px-3 py-1.5 bg-gray-700 border border-gray-600 rounded-lg text-sm"
            />
          </div>

          {/* Min Rating */}
          <div className="flex items-center gap-2">
            <label className="text-sm text-gray-400">Min rating:</label>
            <select
              value={params.min_rating ?? ''}
              onChange={(e) =>
                updateParams({
                  min_rating: e.target.value ? parseFloat(e.target.value) : undefined,
                })
              }
              className="px-3 py-1.5 bg-gray-700 border border-gray-600 rounded-lg text-sm"
            >
              <option value="">Any</option>
              <option value="3">3+</option>
              <option value="4">4+</option>
              <option value="4.5">4.5+</option>
            </select>
          </div>

          {/* Sort buttons */}
          <div className="flex items-center gap-2 ml-auto">
            <span className="text-sm text-gray-400">Sort:</span>
            <button
              onClick={() => toggleSort('count')}
              className={`px-3 py-1.5 rounded-lg text-sm flex items-center gap-1 transition-colors ${
                params.sort_by === 'count'
                  ? 'bg-orange-600 text-white'
                  : 'bg-gray-700 hover:bg-gray-600'
              }`}
            >
              <Image size={14} />
              Count
              {params.sort_by === 'count' && <SortIcon size={14} />}
            </button>
            <button
              onClick={() => toggleSort('rating')}
              className={`px-3 py-1.5 rounded-lg text-sm flex items-center gap-1 transition-colors ${
                params.sort_by === 'rating'
                  ? 'bg-orange-600 text-white'
                  : 'bg-gray-700 hover:bg-gray-600'
              }`}
            >
              <Star size={14} />
              Rating
              {params.sort_by === 'rating' && <SortIcon size={14} />}
            </button>
            <button
              onClick={() => toggleSort('name')}
              className={`px-3 py-1.5 rounded-lg text-sm flex items-center gap-1 transition-colors ${
                params.sort_by === 'name'
                  ? 'bg-orange-600 text-white'
                  : 'bg-gray-700 hover:bg-gray-600'
              }`}
            >
              Name
              {params.sort_by === 'name' && <SortIcon size={14} />}
            </button>
          </div>
        </div>
      </div>

      {/* Loading */}
      {isLoading && (
        <div className="flex items-center justify-center py-16">
          <div className="text-gray-400">Loading LoRAs...</div>
        </div>
      )}

      {/* Error */}
      {error && (
        <div className="flex items-center justify-center py-16">
          <div className="text-red-400">Failed to load LoRAs</div>
        </div>
      )}

      {/* LoRAs Grid */}
      {data && data.items.length > 0 && (
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
          {data.items.map((lora) => (
            <div
              key={lora.name}
              onClick={() => navigate(`/loras/${encodeURIComponent(lora.name)}`)}
              className="bg-gray-800 rounded-lg p-4 hover:bg-gray-750 cursor-pointer transition-colors border border-gray-700 hover:border-gray-600"
            >
              {/* LoRA Name */}
              <h3 className="font-medium text-white truncate mb-2" title={lora.display_name}>
                {lora.display_name}
              </h3>

              {/* Hash indicator */}
              {lora.hash && (
                <span
                  className="inline-block px-2 py-0.5 bg-green-600/20 text-green-400 text-xs rounded mb-3"
                  title={lora.hash}
                >
                  Has hash
                </span>
              )}

              {/* Stats */}
              <div className="flex items-center gap-4 text-sm text-gray-400">
                <div className="flex items-center gap-1">
                  <Image size={14} />
                  <span>{lora.image_count}</span>
                </div>
                {lora.avg_rating !== null && (
                  <div className="flex items-center gap-1">
                    <Star size={14} className="text-yellow-400" />
                    <span>{lora.avg_rating.toFixed(2)}</span>
                  </div>
                )}
                {lora.high_rated_count > 0 && (
                  <div className="text-green-400">{lora.high_rated_count} high</div>
                )}
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Empty State */}
      {data && data.items.length === 0 && (
        <div className="flex flex-col items-center justify-center py-16 text-gray-400">
          <Layers size={48} className="mb-4 opacity-50" />
          <p>No LoRAs found</p>
        </div>
      )}

      {/* Pagination */}
      {data && data.total > (params.limit || 50) && (
        <div className="flex items-center justify-center gap-4">
          <button
            onClick={() =>
              updateParams({ offset: Math.max(0, (params.offset || 0) - (params.limit || 50)) })
            }
            disabled={(params.offset || 0) === 0}
            className="px-4 py-2 bg-gray-700 hover:bg-gray-600 disabled:opacity-50 disabled:cursor-not-allowed rounded-lg transition-colors"
          >
            Previous
          </button>
          <span className="text-gray-400">
            {Math.floor((params.offset || 0) / (params.limit || 50)) + 1} /{' '}
            {Math.ceil(data.total / (params.limit || 50))}
          </span>
          <button
            onClick={() => updateParams({ offset: (params.offset || 0) + (params.limit || 50) })}
            disabled={(params.offset || 0) + (params.limit || 50) >= data.total}
            className="px-4 py-2 bg-gray-700 hover:bg-gray-600 disabled:opacity-50 disabled:cursor-not-allowed rounded-lg transition-colors"
          >
            Next
          </button>
        </div>
      )}
    </div>
  )
}
