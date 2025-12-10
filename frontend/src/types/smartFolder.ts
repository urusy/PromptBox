import type { SearchFilters } from './searchPreset'

export interface SmartFolder {
  id: string
  name: string
  icon: string | null
  filters: SearchFilters
  created_at: string
  updated_at: string
}

export interface SmartFolderCreate {
  name: string
  icon?: string | null
  filters: SearchFilters
}

export interface SmartFolderUpdate {
  name?: string
  icon?: string | null
  filters?: SearchFilters
}
