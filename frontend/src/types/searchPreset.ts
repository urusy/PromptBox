export interface SearchFilters {
  q?: string
  source_tool?: string
  model_type?: string
  model_name?: string
  sampler_name?: string
  min_rating?: number
  exact_rating?: number
  is_favorite?: boolean
  needs_improvement?: boolean
  tags?: string[]
  lora_name?: string
  is_xyz_grid?: boolean | null
  is_upscaled?: boolean | null
  orientation?: string
  min_width?: number
  min_height?: number
  date_from?: string
  sort_by?: string
  sort_order?: 'asc' | 'desc'
}

export interface SearchPreset {
  id: string
  name: string
  filters: SearchFilters
  created_at: string
  updated_at: string
}

export interface SearchPresetCreate {
  name: string
  filters: SearchFilters
}

export interface SearchPresetUpdate {
  name?: string
  filters?: SearchFilters
}
