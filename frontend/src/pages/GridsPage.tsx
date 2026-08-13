import { useCallback } from 'react'
import { useSearchParams } from 'react-router-dom'
import { useQuery } from '@tanstack/react-query'
import { Search, X, Grid3X3, Grid2X2, LayoutGrid } from 'lucide-react'
import { imagesApi } from '@/api/images'
import type { ImageSearchParams } from '@/types/image'
import { useGallerySettingsStore, PER_PAGE_OPTIONS } from '@/stores/gallerySettingsStore'
import ImageGrid from '@/components/gallery/ImageGrid'
import Pagination from '@/components/common/Pagination'

const SORT_OPTIONS = [
  { value: 'created_at', label: 'Date Created' },
  { value: 'updated_at', label: 'Date Updated' },
  { value: 'rating', label: 'Rating' },
] as const

/**
 * グリッド画像専用の一覧（/grids）。
 *
 * 通常のギャラリーはグリッドを表示しない（GalleryPage が is_xyz_grid=false を
 * 固定で送る）ため、グリッドはこの画面だけで扱う。検索条件は通常の一覧ほど
 * 要らないので、キーワード・並び順・表示件数に絞っている。
 */
export default function GridsPage() {
  const [searchParams, setSearchParams] = useSearchParams()
  const { perPage, gridSize, layout, setPerPage, setGridSize } = useGallerySettingsStore()

  const page = Number(searchParams.get('page')) || 1
  const q = searchParams.get('q') || ''
  const sortBy = searchParams.get('sort_by') || 'created_at'

  const params: ImageSearchParams = {
    is_xyz_grid: true,
    q: q || undefined,
    page,
    per_page: perPage,
    sort_by: sortBy,
    sort_order: 'desc',
  }

  const { data, isLoading, error } = useQuery({
    queryKey: ['images', params],
    queryFn: () => imagesApi.list(params),
  })

  const updateParam = useCallback(
    (key: string, value: string) => {
      const next = new URLSearchParams(searchParams)
      if (value) {
        next.set(key, value)
      } else {
        next.delete(key)
      }
      // 条件が変わればページは先頭に戻す。
      if (key !== 'page') {
        next.delete('page')
      }
      setSearchParams(next, { replace: true })
    },
    [searchParams, setSearchParams]
  )

  const handlePageChange = (nextPage: number) => {
    updateParam('page', String(nextPage))
    window.scrollTo({ top: 0, behavior: 'smooth' })
  }

  if (error) {
    return (
      <div className="text-center text-red-500 py-8">Failed to load grids. Please try again.</div>
    )
  }

  return (
    <div>
      <div className="mb-6">
        <h1 className="text-xl font-bold mb-1">Grids</h1>
        <p className="text-sm text-gray-400">
          XYZ プロットなどのグリッド画像。詳細画面から構成画像へ移動できます。
        </p>
      </div>

      <div className="flex flex-wrap items-center gap-2 mb-6">
        <div className="flex-1 min-w-[200px] relative">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400" size={18} />
          <input
            type="text"
            placeholder="Search prompts..."
            defaultValue={q}
            onKeyDown={(e) => {
              if (e.key === 'Enter') {
                updateParam('q', (e.target as HTMLInputElement).value)
              }
            }}
            className="w-full pl-10 pr-4 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>

        {q && (
          <button
            type="button"
            onClick={() => updateParam('q', '')}
            className="p-2 text-gray-400 hover:text-white transition-colors"
            title="Clear search"
          >
            <X size={18} />
          </button>
        )}

        <select
          value={sortBy}
          onChange={(e) => updateParam('sort_by', e.target.value)}
          className="px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-blue-500 text-sm"
        >
          {SORT_OPTIONS.map((option) => (
            <option key={option.value} value={option.value}>
              {option.label}
            </option>
          ))}
        </select>
      </div>

      {isLoading ? (
        <div className="flex items-center justify-center h-64">
          <div className="animate-spin rounded-full h-12 w-12 border-t-2 border-b-2 border-blue-500"></div>
        </div>
      ) : data ? (
        <>
          <div className="flex items-center justify-between mb-4">
            <p className="text-sm text-gray-400">
              {data.total} {data.total === 1 ? 'grid' : 'grids'} found
            </p>
            <div className="flex items-center gap-2">
              <select
                value={perPage}
                onChange={(e) => setPerPage(Number(e.target.value) as typeof perPage)}
                className="bg-gray-800 text-gray-300 text-sm rounded-lg px-2 py-1.5 border-none focus:ring-1 focus:ring-blue-500 cursor-pointer"
                title="Grids per page"
              >
                {PER_PAGE_OPTIONS.map((option) => (
                  <option key={option} value={option}>
                    {option}
                  </option>
                ))}
              </select>
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
            </div>
          </div>

          {/* グリッドは縦横比がまちまちなので、既定は切り抜かない Justified 表示。 */}
          <ImageGrid
            images={data.items}
            size={gridSize}
            layout={layout === 'square' ? 'justified' : layout}
            linkBase="/grids"
          />

          <Pagination page={page} totalPages={data.total_pages} onPageChange={handlePageChange} />
        </>
      ) : null}
    </div>
  )
}
