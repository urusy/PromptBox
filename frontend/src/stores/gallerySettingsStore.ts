import { create } from 'zustand'
import { persist } from 'zustand/middleware'
import type { GridSize } from '@/components/gallery/ImageGrid'

export const PER_PAGE_OPTIONS = [24, 48, 72, 96, 120] as const
export type PerPageOption = typeof PER_PAGE_OPTIONS[number]

interface GallerySettingsState {
  perPage: PerPageOption
  gridSize: GridSize
  setPerPage: (perPage: PerPageOption) => void
  setGridSize: (size: GridSize) => void
}

export const useGallerySettingsStore = create<GallerySettingsState>()(
  persist(
    (set) => ({
      perPage: 48,
      gridSize: 'medium',

      setPerPage: (perPage: PerPageOption) => set({ perPage }),
      setGridSize: (gridSize: GridSize) => set({ gridSize }),
    }),
    {
      name: 'gallery-settings',
    }
  )
)
