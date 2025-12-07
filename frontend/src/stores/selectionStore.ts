import { create } from 'zustand'

interface SelectionState {
  selectedIds: Set<string>
  isSelectionMode: boolean
  toggleSelection: (id: string) => void
  selectAll: (ids: string[]) => void
  clearSelection: () => void
  setSelectionMode: (enabled: boolean) => void
}

export const useSelectionStore = create<SelectionState>((set) => ({
  selectedIds: new Set(),
  isSelectionMode: false,

  toggleSelection: (id: string) =>
    set((state) => {
      const newSet = new Set(state.selectedIds)
      if (newSet.has(id)) {
        newSet.delete(id)
      } else {
        newSet.add(id)
      }
      return { selectedIds: newSet }
    }),

  selectAll: (ids: string[]) =>
    set(() => ({
      selectedIds: new Set(ids),
    })),

  clearSelection: () =>
    set(() => ({
      selectedIds: new Set(),
      isSelectionMode: false,
    })),

  setSelectionMode: (enabled: boolean) =>
    set((state) => ({
      isSelectionMode: enabled,
      selectedIds: enabled ? state.selectedIds : new Set(),
    })),
}))
