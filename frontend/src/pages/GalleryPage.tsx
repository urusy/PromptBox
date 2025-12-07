import { useState, useEffect } from 'react'
import { useQuery } from '@tanstack/react-query'
import { CheckSquare } from 'lucide-react'
import { imagesApi } from '@/api/images'
import type { ImageSearchParams } from '@/types/image'
import { useSelectionStore } from '@/stores/selectionStore'
import SearchForm from '@/components/gallery/SearchForm'
import ImageGrid from '@/components/gallery/ImageGrid'
import Pagination from '@/components/common/Pagination'
import SelectionToolbar from '@/components/gallery/SelectionToolbar'

export default function GalleryPage() {
  const [params, setParams] = useState<ImageSearchParams>({
    page: 1,
    per_page: 24,
    sort_by: 'created_at',
    sort_order: 'desc',
  })

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

  const handleSearch = (newParams: ImageSearchParams) => {
    setParams(newParams)
    clearSelection()
  }

  const handlePageChange = (page: number) => {
    setParams({ ...params, page })
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

          <ImageGrid images={data.items} />

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
