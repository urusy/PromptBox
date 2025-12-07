import { ChevronLeft, ChevronRight } from 'lucide-react'
import clsx from 'clsx'

interface PaginationProps {
  page: number
  totalPages: number
  onPageChange: (page: number) => void
}

export default function Pagination({
  page,
  totalPages,
  onPageChange,
}: PaginationProps) {
  if (totalPages <= 1) return null

  const getPageNumbers = () => {
    const pages: (number | string)[] = []
    const showEllipsis = totalPages > 7

    if (!showEllipsis) {
      for (let i = 1; i <= totalPages; i++) {
        pages.push(i)
      }
    } else {
      pages.push(1)

      if (page > 3) {
        pages.push('...')
      }

      const start = Math.max(2, page - 1)
      const end = Math.min(totalPages - 1, page + 1)

      for (let i = start; i <= end; i++) {
        pages.push(i)
      }

      if (page < totalPages - 2) {
        pages.push('...')
      }

      pages.push(totalPages)
    }

    return pages
  }

  return (
    <div className="flex items-center justify-center gap-2 mt-8">
      <button
        onClick={() => onPageChange(page - 1)}
        disabled={page === 1}
        className={clsx(
          'p-2 rounded-lg transition-colors',
          page === 1
            ? 'text-gray-600 cursor-not-allowed'
            : 'text-gray-400 hover:bg-gray-800 hover:text-white'
        )}
      >
        <ChevronLeft size={20} />
      </button>

      {getPageNumbers().map((p, i) =>
        typeof p === 'number' ? (
          <button
            key={i}
            onClick={() => onPageChange(p)}
            className={clsx(
              'min-w-[40px] h-10 rounded-lg transition-colors',
              p === page
                ? 'bg-blue-600 text-white'
                : 'text-gray-400 hover:bg-gray-800 hover:text-white'
            )}
          >
            {p}
          </button>
        ) : (
          <span key={i} className="px-2 text-gray-600">
            {p}
          </span>
        )
      )}

      <button
        onClick={() => onPageChange(page + 1)}
        disabled={page === totalPages}
        className={clsx(
          'p-2 rounded-lg transition-colors',
          page === totalPages
            ? 'text-gray-600 cursor-not-allowed'
            : 'text-gray-400 hover:bg-gray-800 hover:text-white'
        )}
      >
        <ChevronRight size={20} />
      </button>
    </div>
  )
}
