import { useState, useEffect } from 'react'
import type { GridSize } from '@/components/gallery/ImageGrid'

// Breakpoint definitions (matching Tailwind's breakpoints)
const BREAKPOINTS = {
  sm: 640,
  md: 768,
  lg: 1024,
  xl: 1280,
  '2xl': 1536,
  '3xl': 1920,
  '4xl': 2560,
  '5xl': 3200,
}

// Column counts for each size and breakpoint
const COLUMN_COUNTS: Record<GridSize, Record<string, number>> = {
  small: {
    base: 3,
    sm: 4,
    md: 5,
    lg: 6,
    xl: 8,
    '2xl': 10,
    '3xl': 12,
    '4xl': 14,
    '5xl': 16,
  },
  medium: {
    base: 2,
    sm: 3,
    md: 4,
    lg: 5,
    xl: 6,
    '2xl': 8,
    '3xl': 10,
    '4xl': 12,
    '5xl': 14,
  },
  large: {
    base: 1,
    sm: 2,
    md: 3,
    lg: 4,
    xl: 4,
    '2xl': 5,
    '3xl': 6,
    '4xl': 8,
    '5xl': 10,
  },
}

function getColumnCount(width: number, size: GridSize): number {
  const counts = COLUMN_COUNTS[size]

  if (width >= BREAKPOINTS['5xl']) return counts['5xl']
  if (width >= BREAKPOINTS['4xl']) return counts['4xl']
  if (width >= BREAKPOINTS['3xl']) return counts['3xl']
  if (width >= BREAKPOINTS['2xl']) return counts['2xl']
  if (width >= BREAKPOINTS.xl) return counts.xl
  if (width >= BREAKPOINTS.lg) return counts.lg
  if (width >= BREAKPOINTS.md) return counts.md
  if (width >= BREAKPOINTS.sm) return counts.sm
  return counts.base
}

export function useColumnCount(size: GridSize): number {
  const [columnCount, setColumnCount] = useState(() =>
    typeof window !== 'undefined' ? getColumnCount(window.innerWidth, size) : COLUMN_COUNTS[size].base
  )

  useEffect(() => {
    const handleResize = () => {
      setColumnCount(getColumnCount(window.innerWidth, size))
    }

    window.addEventListener('resize', handleResize)
    return () => window.removeEventListener('resize', handleResize)
  }, [size])

  return columnCount
}
