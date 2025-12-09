import { useState, useEffect, useCallback, useRef } from 'react'
import {
  X,
  Play,
  Pause,
  ChevronLeft,
  ChevronRight,
  Shuffle,
  List,
  Settings,
} from 'lucide-react'
import type { ImageListItem } from '@/types/image'

interface SlideshowProps {
  images: ImageListItem[]
  startIndex?: number
  onClose: () => void
}

export default function Slideshow({ images, startIndex = 0, onClose }: SlideshowProps) {
  const [currentIndex, setCurrentIndex] = useState(startIndex)
  const [isPlaying, setIsPlaying] = useState(true)
  const [isRandom, setIsRandom] = useState(false)
  const [interval, setInterval_] = useState(5) // seconds
  const [showControls, setShowControls] = useState(true)
  const [showSettings, setShowSettings] = useState(false)
  const timerRef = useRef<number | null>(null)
  const controlsTimerRef = useRef<number | null>(null)
  const [randomOrder, setRandomOrder] = useState<number[]>([])
  const [randomPosition, setRandomPosition] = useState(0)

  // Generate random order when random mode is enabled
  useEffect(() => {
    if (isRandom) {
      const indices = Array.from({ length: images.length }, (_, i) => i)
      // Fisher-Yates shuffle
      for (let i = indices.length - 1; i > 0; i--) {
        const j = Math.floor(Math.random() * (i + 1))
        ;[indices[i], indices[j]] = [indices[j], indices[i]]
      }
      setRandomOrder(indices)
      setRandomPosition(0)
    }
  }, [isRandom, images.length])

  const goToNext = useCallback(() => {
    if (isRandom) {
      const nextPos = (randomPosition + 1) % images.length
      setRandomPosition(nextPos)
      setCurrentIndex(randomOrder[nextPos] ?? 0)
    } else {
      setCurrentIndex((prev) => (prev + 1) % images.length)
    }
  }, [isRandom, randomPosition, randomOrder, images.length])

  const goToPrev = useCallback(() => {
    if (isRandom) {
      const prevPos = (randomPosition - 1 + images.length) % images.length
      setRandomPosition(prevPos)
      setCurrentIndex(randomOrder[prevPos] ?? 0)
    } else {
      setCurrentIndex((prev) => (prev - 1 + images.length) % images.length)
    }
  }, [isRandom, randomPosition, randomOrder, images.length])

  // Auto-play timer
  useEffect(() => {
    if (isPlaying && images.length > 1) {
      timerRef.current = window.setTimeout(goToNext, interval * 1000)
    }
    return () => {
      if (timerRef.current) {
        clearTimeout(timerRef.current)
      }
    }
  }, [isPlaying, currentIndex, interval, goToNext, images.length])

  // Hide controls after inactivity
  useEffect(() => {
    const resetControlsTimer = () => {
      setShowControls(true)
      if (controlsTimerRef.current) {
        clearTimeout(controlsTimerRef.current)
      }
      controlsTimerRef.current = window.setTimeout(() => {
        if (!showSettings) {
          setShowControls(false)
        }
      }, 3000)
    }

    resetControlsTimer()
    window.addEventListener('mousemove', resetControlsTimer)
    window.addEventListener('touchstart', resetControlsTimer)

    return () => {
      if (controlsTimerRef.current) {
        clearTimeout(controlsTimerRef.current)
      }
      window.removeEventListener('mousemove', resetControlsTimer)
      window.removeEventListener('touchstart', resetControlsTimer)
    }
  }, [showSettings])

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
          setIsPlaying((prev) => !prev)
          break
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [onClose, goToPrev, goToNext])

  const currentImage = images[currentIndex]
  if (!currentImage) return null

  return (
    <div
      className="fixed inset-0 z-50 bg-black flex items-center justify-center"
      onClick={() => {
        if (!showSettings) {
          onClose()
        }
      }}
    >
      {/* Image */}
      <img
        src={`/storage/${currentImage.storage_path}`}
        alt=""
        className="max-w-full max-h-full object-contain"
        onClick={(e) => e.stopPropagation()}
      />

      {/* Controls overlay */}
      <div
        className={`absolute inset-0 transition-opacity duration-300 ${
          showControls ? 'opacity-100' : 'opacity-0 pointer-events-none'
        }`}
        onClick={(e) => e.stopPropagation()}
      >
        {/* Top bar */}
        <div className="absolute top-0 left-0 right-0 bg-gradient-to-b from-black/70 to-transparent p-4">
          <div className="flex items-center justify-between">
            <div className="text-white text-sm">
              {currentIndex + 1} / {images.length}
            </div>
            <button
              onClick={onClose}
              className="p-2 text-white hover:text-gray-300 transition-colors"
              title="Close (Esc)"
            >
              <X size={24} />
            </button>
          </div>
        </div>

        {/* Bottom bar */}
        <div className="absolute bottom-0 left-0 right-0 bg-gradient-to-t from-black/70 to-transparent p-4">
          <div className="flex items-center justify-center gap-4">
            {/* Previous */}
            <button
              onClick={goToPrev}
              className="p-3 text-white hover:text-gray-300 transition-colors"
              title="Previous (←)"
            >
              <ChevronLeft size={32} />
            </button>

            {/* Play/Pause */}
            <button
              onClick={() => setIsPlaying(!isPlaying)}
              className="p-3 text-white hover:text-gray-300 transition-colors"
              title={isPlaying ? 'Pause (Space)' : 'Play (Space)'}
            >
              {isPlaying ? <Pause size={32} /> : <Play size={32} />}
            </button>

            {/* Next */}
            <button
              onClick={goToNext}
              className="p-3 text-white hover:text-gray-300 transition-colors"
              title="Next (→)"
            >
              <ChevronRight size={32} />
            </button>

            {/* Separator */}
            <div className="w-px h-8 bg-white/30 mx-2" />

            {/* Random toggle */}
            <button
              onClick={() => setIsRandom(!isRandom)}
              className={`p-3 transition-colors ${
                isRandom ? 'text-blue-400' : 'text-white hover:text-gray-300'
              }`}
              title={isRandom ? 'Random: On' : 'Random: Off'}
            >
              {isRandom ? <Shuffle size={24} /> : <List size={24} />}
            </button>

            {/* Settings */}
            <button
              onClick={() => setShowSettings(!showSettings)}
              className={`p-3 transition-colors ${
                showSettings ? 'text-blue-400' : 'text-white hover:text-gray-300'
              }`}
              title="Settings"
            >
              <Settings size={24} />
            </button>
          </div>

          {/* Settings panel */}
          {showSettings && (
            <div className="mt-4 bg-gray-900/90 rounded-lg p-4 max-w-xs mx-auto">
              <div className="flex items-center justify-between gap-4">
                <span className="text-white text-sm">Interval</span>
                <div className="flex items-center gap-2">
                  <input
                    type="range"
                    min={2}
                    max={15}
                    value={interval}
                    onChange={(e) => setInterval_(Number(e.target.value))}
                    className="w-24 accent-blue-500"
                  />
                  <span className="text-white text-sm w-8">{interval}s</span>
                </div>
              </div>
            </div>
          )}
        </div>

        {/* Side navigation areas */}
        <button
          onClick={goToPrev}
          className="absolute left-0 top-1/2 -translate-y-1/2 h-1/2 w-16 flex items-center justify-start pl-2 text-white/50 hover:text-white transition-colors"
        >
          <ChevronLeft size={48} />
        </button>
        <button
          onClick={goToNext}
          className="absolute right-0 top-1/2 -translate-y-1/2 h-1/2 w-16 flex items-center justify-end pr-2 text-white/50 hover:text-white transition-colors"
        >
          <ChevronRight size={48} />
        </button>
      </div>
    </div>
  )
}
