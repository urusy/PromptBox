import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { imagesApi } from '@/api/images'
import type { ImageSearchParams } from '@/types/image'
import SearchForm from '@/components/gallery/SearchForm'
import ImageGrid from '@/components/gallery/ImageGrid'
import Pagination from '@/components/common/Pagination'

export default function GalleryPage() {
  const [params, setParams] = useState<ImageSearchParams>({
    page: 1,
    per_page: 24,
    sort_by: 'created_at',
    sort_order: 'desc',
  })

  const { data, isLoading, error } = useQuery({
    queryKey: ['images', params],
    queryFn: () => imagesApi.list(params),
  })

  const handleSearch = (newParams: ImageSearchParams) => {
    setParams(newParams)
  }

  const handlePageChange = (page: number) => {
    setParams({ ...params, page })
    window.scrollTo({ top: 0, behavior: 'smooth' })
  }

  if (error) {
    return (
      <div className="text-center text-red-500 py-8">
        Failed to load images. Please try again.
      </div>
    )
  }

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
          </div>

          <ImageGrid images={data.items} />

          <Pagination
            page={params.page || 1}
            totalPages={data.total_pages}
            onPageChange={handlePageChange}
          />
        </>
      ) : null}
    </div>
  )
}
