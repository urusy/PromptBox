import { create } from 'zustand'
import { persist } from 'zustand/middleware'
import type { GridSize, GalleryLayout } from '@/components/gallery/ImageGrid'

export const PER_PAGE_OPTIONS = [24, 48, 72, 96, 120] as const
export type PerPageOption = (typeof PER_PAGE_OPTIONS)[number]

interface GallerySettingsState {
  perPage: PerPageOption
  gridSize: GridSize
  layout: GalleryLayout
  setPerPage: (perPage: PerPageOption) => void
  setGridSize: (size: GridSize) => void
  setLayout: (layout: GalleryLayout) => void
}

export const useGallerySettingsStore = create<GallerySettingsState>()(
  persist(
    (set) => ({
      perPage: 48,
      gridSize: 'medium',
      layout: 'square',

      setPerPage: (perPage: PerPageOption) => set({ perPage }),
      setGridSize: (gridSize: GridSize) => set({ gridSize }),
      setLayout: (layout: GalleryLayout) => set({ layout }),
    }),
    {
      name: 'gallery-settings',
    }
  )
)
