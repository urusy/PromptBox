// Types for Model and LoRA resources

export interface ModelListItem {
  name: string // Base model name (version removed)
  display_name: string
  model_type: string | null
  image_count: number
  rated_count: number
  avg_rating: number | null
  high_rated_count: number
  version_count: number // Number of versions for this base model
}

export interface ModelListResponse {
  items: ModelListItem[]
  total: number
}

export interface ModelVersionStats {
  name: string // Full model name with version
  display_name: string // Display name with version
  image_count: number
  rated_count: number
  avg_rating: number | null
  high_rated_count: number
  rating_distribution: Record<number, number>
}

export interface ModelDetail {
  name: string // Base model name (version removed)
  display_name: string
  model_type: string | null
  image_count: number // Total across all versions
  rated_count: number
  avg_rating: number | null
  high_rated_count: number
  rating_distribution: Record<number, number>
  top_samplers: Array<{
    name: string
    count: number
    avg_rating: number | null
  }>
  top_loras: Array<{
    name: string
    count: number
    avg_rating: number | null
  }>
  versions: ModelVersionStats[] // Per-version statistics
}

export interface LoraListItem {
  name: string
  display_name: string
  hash: string | null
  image_count: number
  rated_count: number
  avg_rating: number | null
  high_rated_count: number
}

export interface LoraListResponse {
  items: LoraListItem[]
  total: number
}

export interface LoraDetail {
  name: string
  display_name: string
  hash: string | null
  image_count: number
  rated_count: number
  avg_rating: number | null
  high_rated_count: number
  rating_distribution: Record<number, number>
  avg_weight: number | null
  top_models: Array<{
    name: string
    count: number
    avg_rating: number | null
  }>
  top_samplers: Array<{
    name: string
    count: number
    avg_rating: number | null
  }>
}

export interface CivitaiImage {
  url: string
  width: number | null
  height: number | null
  nsfw: boolean
}

export interface CivitaiRecommendedSettings {
  clip_skip: number | null
  steps: number | null
  cfg_scale: number | null
  sampler: string | null
  vae: string | null
  strength: number | null
}

export interface CivitaiVersionInfo {
  version_id: number
  name: string
  description: string | null
  base_model: string | null
  images: CivitaiImage[]
  recommended_settings: CivitaiRecommendedSettings | null
  trigger_words: string[]
  download_url: string | null
  file_size_kb: number | null
  published_at: string | null
}

export interface CivitaiModelInfo {
  civitai_id: number
  name: string
  description: string | null
  type: string
  nsfw: boolean
  creator: string | null
  civitai_url: string | null
  is_exact_match: boolean
  versions: CivitaiVersionInfo[]
}

export interface CivitaiInfoResponse {
  found: boolean
  info: CivitaiModelInfo | null
  error: string | null
}

export interface ModelSearchParams {
  q?: string
  model_type?: string
  min_count?: number
  min_rating?: number
  sort_by?: 'count' | 'rating' | 'name'
  sort_order?: 'asc' | 'desc'
  limit?: number
  offset?: number
}

export interface LoraSearchParams {
  q?: string
  min_count?: number
  min_rating?: number
  sort_by?: 'count' | 'rating' | 'name'
  sort_order?: 'asc' | 'desc'
  limit?: number
  offset?: number
}
