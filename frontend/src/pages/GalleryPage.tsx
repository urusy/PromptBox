import { useState, useEffect, useCallback } from 'react'
import { useSearchParams } from 'react-router-dom'
import { useQuery } from '@tanstack/react-query'
import { CheckSquare, Grid3X3, Grid2X2, LayoutGrid, Play } from 'lucide-react'
import { imagesApi } from '@/api/images'
import type { ImageSearchParams } from '@/types/image'
import { useSelectionStore } from '@/stores/selectionStore'
import { parseSearchParams, toSearchParams } from '@/utils/searchParams'
import SearchForm from '@/components/gallery/SearchForm'
import ImageGrid, { type GridSize } from '@/components/gallery/ImageGrid'
import Pagination from '@/components/common/Pagination'
import SelectionToolbar from '@/components/gallery/SelectionToolbar'
import Slideshow from '@/components/gallery/Slideshow'

export default function GalleryPage() {
  const [searchParams, setSearchParams] = useSearchParams()
  const [params, setParams] = useState<ImageSearchParams>(() => parseSearchParams(searchParams))
  const [gridSize, setGridSize] = useState<GridSize>('medium')
  const [showSlideshow, setShowSlideshow] = useState(false)

  const { isSelectionMode, setSelectionMode, clearSelection } = useSelectionStore()

  const { data, isLoading, error } = useQuery({
    queryKey: ['images', params],
    queryFn: () => imagesApi.list(params),
  })

  // Sync params from URL when navigating back to gallery
  useEffect(() => {
    const urlParams = parseSearchParams(searchParams)
    setParams(urlParams)
  }, [searchParams])

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
              {/* Slideshow Button */}
              <button
                onClick={() => setShowSlideshow(true)}
                disabled={!data?.items.length}
                className="flex items-center gap-2 px-3 py-1.5 rounded-lg text-sm bg-gray-800 text-gray-400 hover:text-white transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                title="Start slideshow"
              >
                <Play size={16} />
                <span className="hidden sm:inline">Slideshow</span>
              </button>
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

      {/* Slideshow */}
      {showSlideshow && data?.items.length && (
        <Slideshow
          images={data.items}
          onClose={() => setShowSlideshow(false)}
        />
      )}
    </div>
  )
}
