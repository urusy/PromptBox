import { useState, useEffect, useCallback, useRef } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useSearchParams, Link } from 'react-router-dom'
import {
  Heart,
  Star,
  X,
  ChevronLeft,
  ChevronRight,
  RotateCcw,
  Settings,
  ThumbsUp,
  AlertTriangle,
} from 'lucide-react'
import { imagesApi } from '@/api/images'
import type { ImageSearchParams } from '@/types/image'

// Touch/swipe detection
interface SwipeState {
  startX: number
  startY: number
  currentX: number
  isDragging: boolean
}

export default function SwipePage() {
  const [searchParams] = useSearchParams()
  const queryClient = useQueryClient()
  const containerRef = useRef<HTMLDivElement>(null)

  // Parse search params for filters
  const filters: ImageSearchParams = {
    exact_rating: 0, // Default to unrated images
    per_page: 50,
    sort_by: searchParams.get('sort_by') || 'created_at',
    sort_order: (searchParams.get('sort_order') as 'asc' | 'desc') || 'desc',
    model_name: searchParams.get('model_name') || undefined,
    model_type: searchParams.get('model_type') || undefined,
    source_tool: searchParams.get('source_tool') || undefined,
  }

  // State
  const [currentIndex, setCurrentIndex] = useState(0)
  const [showSettings, setShowSettings] = useState(false)
  const [swipeState, setSwipeState] = useState<SwipeState>({
    startX: 0,
    startY: 0,
    currentX: 0,
    isDragging: false,
  })
  const [swipeDirection, setSwipeDirection] = useState<'left' | 'right' | null>(null)
  const [recentActions, setRecentActions] = useState<
    Array<{
      imageId: string
      action: 'favorite' | 'rating' | 'skip' | 'improve'
      value?: number
    }>
  >([])

  // Fetch images
  const { data, isLoading, error, refetch } = useQuery({
    queryKey: ['swipe-images', filters],
    queryFn: () => imagesApi.list(filters),
  })

  const images = data?.items || []
  const currentImage = images[currentIndex]

  // Mutation for updating images
  const updateMutation = useMutation({
    mutationFn: ({
      id,
      update,
    }: {
      id: string
      update: { rating?: number; is_favorite?: boolean; needs_improvement?: boolean }
    }) => imagesApi.update(id, update),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['swipe-images'] })
      queryClient.invalidateQueries({ queryKey: ['images'] })
    },
  })

  // Navigation
  const goNext = useCallback(() => {
    if (currentIndex < images.length - 1) {
      setCurrentIndex((prev) => prev + 1)
    }
  }, [currentIndex, images.length])

  const goPrev = useCallback(() => {
    if (currentIndex > 0) {
      setCurrentIndex((prev) => prev - 1)
    }
  }, [currentIndex])

  // Actions
  const handleRating = useCallback(
    (rating: number) => {
      if (!currentImage) return
      updateMutation.mutate({ id: currentImage.id, update: { rating } })
      setRecentActions((prev) => [
        ...prev.slice(-9),
        { imageId: currentImage.id, action: 'rating', value: rating },
      ])
      setSwipeDirection('right')
      setTimeout(() => {
        setSwipeDirection(null)
        goNext()
      }, 300)
    },
    [currentImage, updateMutation, goNext]
  )

  const handleFavorite = useCallback(() => {
    if (!currentImage) return
    updateMutation.mutate({
      id: currentImage.id,
      update: { is_favorite: true, rating: currentImage.rating || 5 },
    })
    setRecentActions((prev) => [
      ...prev.slice(-9),
      { imageId: currentImage.id, action: 'favorite' },
    ])
    setSwipeDirection('right')
    setTimeout(() => {
      setSwipeDirection(null)
      goNext()
    }, 300)
  }, [currentImage, updateMutation, goNext])

  const handleSkip = useCallback(() => {
    if (!currentImage) return
    setRecentActions((prev) => [...prev.slice(-9), { imageId: currentImage.id, action: 'skip' }])
    setSwipeDirection('left')
    setTimeout(() => {
      setSwipeDirection(null)
      goNext()
    }, 300)
  }, [currentImage, goNext])

  const handleNeedsImprovement = useCallback(() => {
    if (!currentImage) return
    updateMutation.mutate({ id: currentImage.id, update: { needs_improvement: true, rating: 1 } })
    setRecentActions((prev) => [...prev.slice(-9), { imageId: currentImage.id, action: 'improve' }])
    setSwipeDirection('left')
    setTimeout(() => {
      setSwipeDirection(null)
      goNext()
    }, 300)
  }, [currentImage, updateMutation, goNext])

  const handleUndo = useCallback(() => {
    if (recentActions.length === 0) return
    // Undo by going back and removing from recent actions
    // Note: This only navigates back, doesn't reverse the mutation
    setRecentActions((prev) => prev.slice(0, -1))
    goPrev()
  }, [recentActions, goPrev])

  // Keyboard navigation
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return

      switch (e.key) {
        case 'ArrowLeft':
          handleSkip()
          break
        case 'ArrowRight':
          handleFavorite()
          break
        case 'ArrowUp':
          goPrev()
          break
        case 'ArrowDown':
          goNext()
          break
        case '1':
        case '2':
        case '3':
        case '4':
        case '5':
          handleRating(parseInt(e.key))
          break
        case 'f':
        case 'F':
          handleFavorite()
          break
        case 's':
        case 'S':
          handleSkip()
          break
        case 'i':
        case 'I':
          handleNeedsImprovement()
          break
        case 'z':
          if (e.ctrlKey || e.metaKey) {
            handleUndo()
          }
          break
        case 'Escape':
          setShowSettings(false)
          break
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [handleSkip, handleFavorite, handleRating, goNext, goPrev, handleNeedsImprovement, handleUndo])

  // Touch handling
  const handleTouchStart = (e: React.TouchEvent) => {
    setSwipeState({
      startX: e.touches[0].clientX,
      startY: e.touches[0].clientY,
      currentX: e.touches[0].clientX,
      isDragging: true,
    })
  }

  const handleTouchMove = (e: React.TouchEvent) => {
    if (!swipeState.isDragging) return
    setSwipeState((prev) => ({
      ...prev,
      currentX: e.touches[0].clientX,
    }))
  }

  const handleTouchEnd = () => {
    if (!swipeState.isDragging) return

    const deltaX = swipeState.currentX - swipeState.startX
    const threshold = 100

    if (deltaX > threshold) {
      handleFavorite()
    } else if (deltaX < -threshold) {
      handleSkip()
    }

    setSwipeState({
      startX: 0,
      startY: 0,
      currentX: 0,
      isDragging: false,
    })
  }

  // Calculate swipe offset for animation
  const swipeOffset = swipeState.isDragging ? swipeState.currentX - swipeState.startX : 0

  if (isLoading) {
    return (
      <div className="flex items-center justify-center min-h-[60vh]">
        <div className="text-gray-400">Loading images...</div>
      </div>
    )
  }

  if (error) {
    return (
      <div className="flex items-center justify-center min-h-[60vh]">
        <div className="text-red-400">Failed to load images</div>
      </div>
    )
  }

  if (images.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center min-h-[60vh] gap-4">
        <Star size={64} className="text-gray-600" />
        <h2 className="text-xl font-semibold text-gray-400">No unrated images found</h2>
        <p className="text-gray-500">All images have been rated!</p>
        <button
          onClick={() => refetch()}
          className="px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded-lg transition-colors"
        >
          Refresh
        </button>
      </div>
    )
  }

  if (currentIndex >= images.length) {
    return (
      <div className="flex flex-col items-center justify-center min-h-[60vh] gap-4">
        <ThumbsUp size={64} className="text-green-500" />
        <h2 className="text-xl font-semibold">All done!</h2>
        <p className="text-gray-400">You've reviewed all {images.length} images in this batch.</p>
        <div className="flex gap-4">
          <button
            onClick={() => {
              setCurrentIndex(0)
              refetch()
            }}
            className="px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded-lg transition-colors"
          >
            Load More
          </button>
          <Link
            to="/"
            className="px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded-lg transition-colors"
          >
            Back to Gallery
          </Link>
        </div>
      </div>
    )
  }

  return (
    <div className="flex flex-col h-[calc(100vh-8rem)] max-h-[800px]">
      {/* Header */}
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-4">
          <h1 className="text-xl font-bold">Quick Rate</h1>
          <span className="text-gray-400 text-sm">
            {currentIndex + 1} / {images.length}
          </span>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={handleUndo}
            disabled={recentActions.length === 0}
            className="p-2 hover:bg-gray-700 rounded-lg transition-colors disabled:opacity-30"
            title="Undo (Ctrl+Z)"
          >
            <RotateCcw size={20} />
          </button>
          <button
            onClick={() => setShowSettings(!showSettings)}
            className="p-2 hover:bg-gray-700 rounded-lg transition-colors"
            title="Settings"
          >
            <Settings size={20} />
          </button>
        </div>
      </div>

      {/* Settings Panel */}
      {showSettings && (
        <div className="bg-gray-800 rounded-lg p-4 mb-4">
          <h3 className="text-sm font-semibold mb-2">Keyboard Shortcuts</h3>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-2 text-sm text-gray-400">
            <div>
              <kbd className="bg-gray-700 px-2 py-1 rounded">1-5</kbd> Rate
            </div>
            <div>
              <kbd className="bg-gray-700 px-2 py-1 rounded">F</kbd> Favorite
            </div>
            <div>
              <kbd className="bg-gray-700 px-2 py-1 rounded">S</kbd> Skip
            </div>
            <div>
              <kbd className="bg-gray-700 px-2 py-1 rounded">I</kbd> Needs Improvement
            </div>
            <div>
              <kbd className="bg-gray-700 px-2 py-1 rounded">←</kbd> Skip
            </div>
            <div>
              <kbd className="bg-gray-700 px-2 py-1 rounded">→</kbd> Favorite
            </div>
            <div>
              <kbd className="bg-gray-700 px-2 py-1 rounded">↑/↓</kbd> Navigate
            </div>
            <div>
              <kbd className="bg-gray-700 px-2 py-1 rounded">Ctrl+Z</kbd> Undo
            </div>
          </div>
        </div>
      )}

      {/* Main Card Area */}
      <div
        ref={containerRef}
        className="flex-1 relative overflow-hidden"
        onTouchStart={handleTouchStart}
        onTouchMove={handleTouchMove}
        onTouchEnd={handleTouchEnd}
      >
        {currentImage && (
          <div
            className={`absolute inset-0 flex items-center justify-center transition-transform duration-300 ${
              swipeDirection === 'left'
                ? '-translate-x-full opacity-0'
                : swipeDirection === 'right'
                  ? 'translate-x-full opacity-0'
                  : ''
            }`}
            style={{
              transform: swipeState.isDragging
                ? `translateX(${swipeOffset}px) rotate(${swipeOffset * 0.02}deg)`
                : undefined,
            }}
          >
            <div className="relative max-w-full max-h-full">
              {/* Swipe indicators */}
              {swipeOffset > 50 && (
                <div className="absolute inset-0 flex items-center justify-center bg-green-500/20 rounded-lg z-10">
                  <Heart size={64} className="text-green-500" />
                </div>
              )}
              {swipeOffset < -50 && (
                <div className="absolute inset-0 flex items-center justify-center bg-red-500/20 rounded-lg z-10">
                  <X size={64} className="text-red-500" />
                </div>
              )}

              {/* Image */}
              <Link to={`/image/${currentImage.id}`}>
                <img
                  src={`/storage/${currentImage.storage_path}`}
                  alt=""
                  className="max-w-full max-h-[60vh] object-contain rounded-lg shadow-2xl"
                  draggable={false}
                />
              </Link>

              {/* Image info overlay */}
              <div className="absolute bottom-0 left-0 right-0 bg-gradient-to-t from-black/80 to-transparent p-4 rounded-b-lg">
                <div className="flex items-center gap-2 text-sm text-gray-300">
                  {currentImage.model_name && (
                    <span className="truncate">{currentImage.model_name}</span>
                  )}
                  <span className="text-gray-500">
                    {currentImage.width}×{currentImage.height}
                  </span>
                </div>
              </div>
            </div>
          </div>
        )}

        {/* Navigation arrows */}
        <button
          onClick={goPrev}
          disabled={currentIndex === 0}
          className="absolute left-2 top-1/2 -translate-y-1/2 p-2 bg-gray-800/80 hover:bg-gray-700 rounded-full transition-colors disabled:opacity-30"
        >
          <ChevronLeft size={24} />
        </button>
        <button
          onClick={goNext}
          disabled={currentIndex >= images.length - 1}
          className="absolute right-2 top-1/2 -translate-y-1/2 p-2 bg-gray-800/80 hover:bg-gray-700 rounded-full transition-colors disabled:opacity-30"
        >
          <ChevronRight size={24} />
        </button>
      </div>

      {/* Action Buttons */}
      <div className="flex items-center justify-center gap-3 mt-4 pb-4">
        {/* Skip / Needs Improvement */}
        <button
          onClick={handleNeedsImprovement}
          className="p-3 bg-orange-600/20 hover:bg-orange-600/40 text-orange-400 rounded-full transition-colors"
          title="Needs Improvement (I)"
        >
          <AlertTriangle size={24} />
        </button>
        <button
          onClick={handleSkip}
          className="p-4 bg-red-600/20 hover:bg-red-600/40 text-red-400 rounded-full transition-colors"
          title="Skip (S)"
        >
          <X size={28} />
        </button>

        {/* Rating buttons */}
        <div className="flex items-center gap-1 mx-2">
          {[1, 2, 3, 4, 5].map((rating) => (
            <button
              key={rating}
              onClick={() => handleRating(rating)}
              className="p-2 hover:bg-gray-700 rounded-lg transition-colors group"
              title={`Rate ${rating} (${rating})`}
            >
              <Star
                size={24}
                className={`${
                  rating <= (currentImage?.rating || 0)
                    ? 'text-yellow-400 fill-yellow-400'
                    : 'text-gray-500 group-hover:text-yellow-400'
                }`}
              />
            </button>
          ))}
        </div>

        {/* Favorite */}
        <button
          onClick={handleFavorite}
          className="p-4 bg-green-600/20 hover:bg-green-600/40 text-green-400 rounded-full transition-colors"
          title="Favorite (F)"
        >
          <Heart size={28} />
        </button>
      </div>

      {/* Progress bar */}
      <div className="w-full bg-gray-700 h-1 rounded-full overflow-hidden">
        <div
          className="bg-blue-500 h-full transition-all duration-300"
          style={{ width: `${((currentIndex + 1) / images.length) * 100}%` }}
        />
      </div>
    </div>
  )
}
