export interface LoraInfo {
  name: string
  weight: number
  weight_clip?: number
  hash?: string
}

export interface ControlNetInfo {
  model: string
  weight: number
  guidance_start: number
  guidance_end: number
  preprocessor?: string
}

export interface EmbeddingInfo {
  name: string
  hash?: string
}

export interface Image {
  id: string
  source_tool: string
  model_type: string | null
  has_metadata: boolean
  original_filename: string
  storage_path: string
  thumbnail_path: string
  file_hash: string
  width: number
  height: number
  file_size_bytes: number
  positive_prompt: string | null
  negative_prompt: string | null
  model_name: string | null
  sampler_name: string | null
  scheduler: string | null
  steps: number | null
  cfg_scale: number | null
  seed: number | null
  loras: LoraInfo[]
  controlnets: ControlNetInfo[]
  embeddings: EmbeddingInfo[]
  model_params: Record<string, unknown>
  workflow_extras: Record<string, unknown>
  raw_metadata: Record<string, unknown> | null
  rating: number
  is_favorite: boolean
  needs_improvement: boolean
  user_tags: string[]
  user_memo: string | null
  created_at: string
  updated_at: string
  deleted_at: string | null
}

export interface ImageListItem {
  id: string
  source_tool: string
  model_type: string | null
  thumbnail_path: string
  width: number
  height: number
  model_name: string | null
  rating: number
  is_favorite: boolean
  created_at: string
}

export interface ImageUpdate {
  rating?: number
  is_favorite?: boolean
  needs_improvement?: boolean
  user_tags?: string[]
  user_memo?: string
}

export interface PaginatedResponse<T> {
  items: T[]
  total: number
  page: number
  per_page: number
  total_pages: number
}

export interface ImageSearchParams {
  q?: string
  source_tool?: string
  model_type?: string
  model_name?: string
  sampler_name?: string
  min_rating?: number
  is_favorite?: boolean
  needs_improvement?: boolean
  tags?: string[]
  lora_name?: string
  is_xyz_grid?: boolean | null  // true = grid only, false = non-grid only, null/undefined = all
  include_deleted?: boolean
  page?: number
  per_page?: number
  sort_by?: string
  sort_order?: 'asc' | 'desc'
}
