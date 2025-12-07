import { Star } from 'lucide-react'
import clsx from 'clsx'

interface StarRatingProps {
  rating: number
  onChange?: (rating: number) => void
  size?: number
  readonly?: boolean
}

export default function StarRating({
  rating,
  onChange,
  size = 20,
  readonly = false,
}: StarRatingProps) {
  const handleClick = (star: number) => {
    if (readonly || !onChange) return
    // Toggle off if clicking the same rating
    onChange(star === rating ? 0 : star)
  }

  return (
    <div className="flex items-center gap-0.5">
      {[1, 2, 3, 4, 5].map((star) => (
        <button
          key={star}
          type="button"
          onClick={() => handleClick(star)}
          disabled={readonly}
          className={clsx(
            'p-0.5 transition-colors',
            !readonly && 'hover:scale-110 cursor-pointer',
            readonly && 'cursor-default'
          )}
        >
          <Star
            size={size}
            className={clsx(
              star <= rating ? 'text-yellow-400' : 'text-gray-600',
              !readonly && 'hover:text-yellow-300'
            )}
            fill={star <= rating ? 'currentColor' : 'none'}
          />
        </button>
      ))}
    </div>
  )
}
