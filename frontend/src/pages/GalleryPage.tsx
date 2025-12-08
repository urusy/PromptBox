import { useState, useEffect, useCallback } from 'react'
import { useSearchParams } from 'react-router-dom'
import { useQuery } from '@tanstack/react-query'
import { CheckSquare, Grid3X3, Grid2X2, LayoutGrid } from 'lucide-react'
import { imagesApi } from '@/api/images'
import type { ImageSearchParams } from '@/types/image'
import { useSelectionStore } from '@/stores/selectionStore'
import SearchForm from '@/components/gallery/SearchForm'
import ImageGrid, { type GridSize } from '@/components/gallery/ImageGrid'
import Pagination from '@/components/common/Pagination'
import SelectionToolbar from '@/components/gallery/SelectionToolbar'

const DEFAULT_PARAMS: ImageSearchParams = {
  page: 1,
  per_page: 24,
  sort_by: 'created_at',
  sort_order: 'desc',
}

// Parse URL search params to ImageSearchParams
function parseSearchParams(searchParams: URLSearchParams): ImageSearchParams {
  const params: ImageSearchParams = { ...DEFAULT_PARAMS }

  const q = searchParams.get('q')
  if (q) params.q = q

  const source_tool = searchParams.get('source_tool')
  if (source_tool) params.source_tool = source_tool

  const model_type = searchParams.get('model_type')
  if (model_type) params.model_type = model_type

  const model_name = searchParams.get('model_name')
  if (model_name) params.model_name = model_name

  const min_rating = searchParams.get('min_rating')
  if (min_rating) params.min_rating = parseInt(min_rating)

  const exact_rating = searchParams.get('exact_rating')
  if (exact_rating) params.exact_rating = parseInt(exact_rating)

  const is_favorite = searchParams.get('is_favorite')
  if (is_favorite === 'true') params.is_favorite = true

  const is_xyz_grid = searchParams.get('is_xyz_grid')
  if (is_xyz_grid === 'true') params.is_xyz_grid = true
  else if (is_xyz_grid === 'false') params.is_xyz_grid = false

  const is_upscaled = searchParams.get('is_upscaled')
  if (is_upscaled === 'true') params.is_upscaled = true
  else if (is_upscaled === 'false') params.is_upscaled = false

  const min_width = searchParams.get('min_width')
  if (min_width) params.min_width = parseInt(min_width)

  const min_height = searchParams.get('min_height')
  if (min_height) params.min_height = parseInt(min_height)

  const page = searchParams.get('page')
  if (page) params.page = parseInt(page)

  const per_page = searchParams.get('per_page')
  if (per_page) params.per_page = parseInt(per_page)

  const sort_by = searchParams.get('sort_by')
  if (sort_by) params.sort_by = sort_by

  const sort_order = searchParams.get('sort_order')
  if (sort_order === 'asc' || sort_order === 'desc') params.sort_order = sort_order

  return params
}

// Convert ImageSearchParams to URLSearchParams
function toSearchParams(params: ImageSearchParams): URLSearchParams {
  const searchParams = new URLSearchParams()

  if (params.q) searchParams.set('q', params.q)
  if (params.source_tool) searchParams.set('source_tool', params.source_tool)
  if (params.model_type) searchParams.set('model_type', params.model_type)
  if (params.model_name) searchParams.set('model_name', params.model_name)
  if (params.min_rating !== undefined) searchParams.set('min_rating', params.min_rating.toString())
  if (params.exact_rating !== undefined) searchParams.set('exact_rating', params.exact_rating.toString())
  if (params.is_favorite === true) searchParams.set('is_favorite', 'true')
  if (params.is_xyz_grid === true) searchParams.set('is_xyz_grid', 'true')
  else if (params.is_xyz_grid === false) searchParams.set('is_xyz_grid', 'false')
  if (params.is_upscaled === true) searchParams.set('is_upscaled', 'true')
  else if (params.is_upscaled === false) searchParams.set('is_upscaled', 'false')
  if (params.min_width) searchParams.set('min_width', params.min_width.toString())
  if (params.min_height) searchParams.set('min_height', params.min_height.toString())
  if (params.page && params.page !== 1) searchParams.set('page', params.page.toString())
  if (params.per_page && params.per_page !== 24) searchParams.set('per_page', params.per_page.toString())
  if (params.sort_by && params.sort_by !== 'created_at') searchParams.set('sort_by', params.sort_by)
  if (params.sort_order && params.sort_order !== 'desc') searchParams.set('sort_order', params.sort_order)

  return searchParams
}

export default function GalleryPage() {
  const [searchParams, setSearchParams] = useSearchParams()
  const [params, setParams] = useState<ImageSearchParams>(() => parseSearchParams(searchParams))
  const [gridSize, setGridSize] = useState<GridSize>('medium')

  const { isSelectionMode, setSelectionMode, clearSelection } = useSelectionStore()

  const { data, isLoading, error } = useQuery({
    queryKey: ['images', params],
    queryFn: () => imagesApi.list(params),
  })

  // Clear selection when navigating away
  useEffect(() => {
    return () => {
      clearSelection()
    }
  }, [clearSelection])

  // Sync URL when params change
  const updateUrl = useCallback((newParams: ImageSearchParams) => {
    setSearchParams(toSearchParams(newParams), { replace: true })
  }, [setSearchParams])

  const handleSearch = (newParams: ImageSearchParams) => {
    setParams(newParams)
    updateUrl(newParams)
    clearSelection()
  }

  const handlePageChange = (page: number) => {
    const newParams = { ...params, page }
    setParams(newParams)
    updateUrl(newParams)
    clearSelection()
    window.scrollTo({ top: 0, behavior: 'smooth' })
  }

  const toggleSelectionMode = () => {
    if (isSelectionMode) {
      clearSelection()
    } else {
      setSelectionMode(true)
    }
  }

  if (error) {
    return (
      <div className="text-center text-red-500 py-8">
        Failed to load images. Please try again.
      </div>
    )
  }

  const allImageIds = data?.items.map((img) => img.id) || []

  return (
    <div>
      <SearchForm params={params} onSearch={handleSearch} />

      {isLoading ? (
        <div className="flex items-center justify-center h-64">
          <div className="animate-spin rounded-full h-12 w-12 border-t-2 border-b-2 border-blue-500"></div>
        </div>
      ) : data ? (
        <>
          <div className="flex items-center justify-between mb-4">
            <p className="text-sm text-gray-400">
              {data.total} {data.total === 1 ? 'image' : 'images'} found
            </p>
            <div className="flex items-center gap-2">
              {/* Grid Size Toggle */}
              <div className="flex items-center bg-gray-800 rounded-lg p-1">
                <button
                  onClick={() => setGridSize('small')}
                  className={`p-1.5 rounded transition-colors ${
                    gridSize === 'small'
                      ? 'bg-gray-700 text-white'
                      : 'text-gray-400 hover:text-white'
                  }`}
                  title="Small thumbnails"
                >
                  <Grid3X3 size={16} />
                </button>
                <button
                  onClick={() => setGridSize('medium')}
                  className={`p-1.5 rounded transition-colors ${
                    gridSize === 'medium'
                      ? 'bg-gray-700 text-white'
                      : 'text-gray-400 hover:text-white'
                  }`}
                  title="Medium thumbnails"
                >
                  <Grid2X2 size={16} />
                </button>
                <button
                  onClick={() => setGridSize('large')}
                  className={`p-1.5 rounded transition-colors ${
                    gridSize === 'large'
                      ? 'bg-gray-700 text-white'
                      : 'text-gray-400 hover:text-white'
                  }`}
                  title="Large thumbnails"
                >
                  <LayoutGrid size={16} />
                </button>
              </div>
              {/* Selection Mode Toggle */}
              <button
                onClick={toggleSelectionMode}
                className={`flex items-center gap-2 px-3 py-1.5 rounded-lg text-sm transition-colors ${
                  isSelectionMode
                    ? 'bg-blue-600 text-white'
                    : 'bg-gray-800 text-gray-400 hover:text-white'
                }`}
              >
                <CheckSquare size={16} />
                {isSelectionMode ? 'Cancel' : 'Select'}
              </button>
            </div>
          </div>

          <ImageGrid images={data.items} size={gridSize} />

          <Pagination
            page={params.page || 1}
            totalPages={data.total_pages}
            onPageChange={handlePageChange}
          />

          <SelectionToolbar totalCount={data.items.length} allIds={allImageIds} />
        </>
      ) : null}
    </div>
  )
}
