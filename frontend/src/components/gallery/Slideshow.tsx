import { useState, useEffect, useCallback } from 'react'
import { X, Play, Pause, ChevronLeft, ChevronRight, Shuffle } from 'lucide-react'

// Slideshow interval options (in seconds)
const INTERVAL_OPTIONS = [3, 5, 10, 15, 30]

// Generic image type - only requires storage_path
interface SlideshowImage {
  storage_path: string
}

interface SlideshowProps {
  images: SlideshowImage[]
  startIndex?: number
  onClose: () => void
}

// Fisher-Yates shuffle algorithm
function shuffleArray<T>(array: T[]): T[] {
  const shuffled = [...array]
  for (let i = shuffled.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1))
    ;[shuffled[i], shuffled[j]] = [shuffled[j], shuffled[i]]
  }
  return shuffled
}

export default function Slideshow({ images, startIndex = 0, onClose }: SlideshowProps) {
  const [currentIndex, setCurrentIndex] = useState(0)
  const [isPlaying, setIsPlaying] = useState(true)
  const [isShuffled, setIsShuffled] = useState(false)
  const [interval, setInterval_] = useState(5)
  const [showControls, setShowControls] = useState(true)
  const [displayOrder, setDisplayOrder] = useState<number[]>(() =>
    Array.from({ length: images.length }, (_, i) => i)
  )

  // Initialize with the correct starting position
  useEffect(() => {
    if (!isShuffled) {
      setDisplayOrder(Array.from({ length: images.length }, (_, i) => i))
      setCurrentIndex(startIndex)
    }
  }, [images.length, startIndex, isShuffled])

  const currentImageIndex = displayOrder[currentIndex]
  const currentImage = images[currentImageIndex]

  const goToNext = useCallback(() => {
    setCurrentIndex((prev) => (prev + 1) % images.length)
  }, [images.length])

  const goToPrev = useCallback(() => {
    setCurrentIndex((prev) => (prev - 1 + images.length) % images.length)
  }, [images.length])

  const togglePlay = useCallback(() => {
    setIsPlaying((prev) => !prev)
  }, [])

  const toggleShuffle = useCallback(() => {
    setIsShuffled((prev) => {
      if (!prev) {
        // Turn on shuffle: create shuffled order starting from current image
        const indices = Array.from({ length: images.length }, (_, i) => i)
        const currentImgIdx = displayOrder[currentIndex]
        const otherIndices = indices.filter((i) => i !== currentImgIdx)
        const shuffledOthers = shuffleArray(otherIndices)
        setDisplayOrder([currentImgIdx, ...shuffledOthers])
        setCurrentIndex(0)
      } else {
        // Turn off shuffle: restore original order, find current image position
        const currentImgIdx = displayOrder[currentIndex]
        setDisplayOrder(Array.from({ length: images.length }, (_, i) => i))
        setCurrentIndex(currentImgIdx)
      }
      return !prev
    })
  }, [images.length, displayOrder, currentIndex])

  // Auto-advance slideshow
  useEffect(() => {
    if (!isPlaying || images.length <= 1) return

    const timer = window.setInterval(() => {
      goToNext()
    }, interval * 1000)

    return () => window.clearInterval(timer)
  }, [isPlaying, interval, goToNext, images.length])

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      switch (e.key) {
        case 'Escape':
          onClose()
          break
        case 'ArrowLeft':
          goToPrev()
          break
        case 'ArrowRight':
          goToNext()
          break
        case ' ':
          e.preventDefault()
          togglePlay()
          break
        case 's':
        case 'S':
          toggleShuffle()
          break
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [onClose, goToNext, goToPrev, togglePlay, toggleShuffle])

  // Prevent body scroll when slideshow is open (iOS Safari fix)
  useEffect(() => {
    document.body.style.overflow = 'hidden'
    return () => {
      document.body.style.overflow = ''
    }
  }, [])

  // Auto-hide controls
  useEffect(() => {
    let hideTimer: number

    const resetHideTimer = () => {
      setShowControls(true)
      window.clearTimeout(hideTimer)
      hideTimer = window.setTimeout(() => {
        if (isPlaying) {
          setShowControls(false)
        }
      }, 3000)
    }

    resetHideTimer()

    const handleMouseMove = () => resetHideTimer()
    const handleTouchStart = () => resetHideTimer()
    window.addEventListener('mousemove', handleMouseMove)
    window.addEventListener('touchstart', handleTouchStart)

    return () => {
      window.clearTimeout(hideTimer)
      window.removeEventListener('mousemove', handleMouseMove)
      window.removeEventListener('touchstart', handleTouchStart)
    }
  }, [isPlaying])

  if (!currentImage) return null

  return (
    <div className="fixed inset-0 bg-black z-50 flex items-center justify-center">
      {/* Main image */}
      <img
        src={`/storage/${currentImage.storage_path}`}
        alt=""
        className="max-w-full max-h-full object-contain"
      />

      {/* Controls overlay */}
      <div
        className={`absolute inset-0 transition-opacity duration-300 ${
          showControls ? 'opacity-100' : 'opacity-0 pointer-events-none'
        }`}
      >
        {/* Top bar */}
        <div className="absolute top-0 left-0 right-0 p-4 bg-gradient-to-b from-black/70 to-transparent">
          <div className="flex items-center justify-between">
            <div className="text-white text-sm">
              {currentIndex + 1} / {images.length}
            </div>
            <div className="flex items-center gap-3">
              {/* Shuffle button */}
              <button
                onClick={toggleShuffle}
                className={`p-2 rounded-lg transition-colors ${
                  isShuffled
                    ? 'bg-green-500 text-white'
                    : 'bg-white/20 text-white hover:bg-white/30'
                }`}
                title={isShuffled ? 'シャッフル: オン (S)' : 'シャッフル: オフ (S)'}
              >
                <Shuffle size={20} />
              </button>
              {/* Interval selector */}
              <select
                value={interval}
                onChange={(e) => setInterval_(Number(e.target.value))}
                className="bg-white/20 text-white text-sm px-3 py-2 rounded-lg border-none cursor-pointer hover:bg-white/30 transition-colors"
              >
                {INTERVAL_OPTIONS.map((sec) => (
                  <option key={sec} value={sec} className="bg-gray-800 text-white">
                    {sec}秒
                  </option>
                ))}
              </select>
              {/* Close button */}
              <button
                onClick={onClose}
                className="p-2 bg-white/20 text-white hover:bg-white/30 rounded-lg transition-colors"
                title="閉じる (Esc)"
              >
                <X size={20} />
              </button>
            </div>
          </div>
        </div>

        {/* Bottom bar */}
        <div className="absolute bottom-0 left-0 right-0 p-4 bg-gradient-to-t from-black/70 to-transparent">
          <div className="flex items-center justify-center gap-4">
            {/* Prev button */}
            <button
              onClick={goToPrev}
              className="p-3 text-white hover:bg-white/20 rounded-full transition-colors"
              title="前へ (←)"
            >
              <ChevronLeft size={32} />
            </button>
            {/* Play/Pause button */}
            <button
              onClick={togglePlay}
              className="p-4 text-white bg-white/20 hover:bg-white/30 rounded-full transition-colors"
              title={isPlaying ? '一時停止 (Space)' : '再生 (Space)'}
            >
              {isPlaying ? <Pause size={32} /> : <Play size={32} />}
            </button>
            {/* Next button */}
            <button
              onClick={goToNext}
              className="p-3 text-white hover:bg-white/20 rounded-full transition-colors"
              title="次へ (→)"
            >
              <ChevronRight size={32} />
            </button>
          </div>
        </div>

        {/* Side navigation areas */}
        <button
          onClick={goToPrev}
          className="absolute left-0 top-1/2 -translate-y-1/2 w-20 h-1/2 flex items-center justify-start pl-4 text-white/50 hover:text-white transition-colors"
        >
          <ChevronLeft size={48} />
        </button>
        <button
          onClick={goToNext}
          className="absolute right-0 top-1/2 -translate-y-1/2 w-20 h-1/2 flex items-center justify-end pr-4 text-white/50 hover:text-white transition-colors"
        >
          <ChevronRight size={48} />
        </button>
      </div>

      {/* Progress bar */}
      {isPlaying && (
        <div className="absolute bottom-0 left-0 right-0 h-1 bg-white/20">
          <div
            className="h-full bg-white/70"
            style={{
              animation: `slideshow-progress ${interval}s linear infinite`,
            }}
          />
        </div>
      )}

      {/* CSS for progress animation */}
      <style>{`
        @keyframes slideshow-progress {
          from { width: 0%; }
          to { width: 100%; }
        }
      `}</style>
    </div>
  )
}
