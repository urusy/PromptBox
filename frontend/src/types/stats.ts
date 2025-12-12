export interface CountItem {
  name: string
  count: number
}

export interface TimeSeriesItem {
  date: string
  count: number
}

export interface RatingDistribution {
  rating: number
  count: number
}

export interface StatsOverview {
  total_images: number
  total_favorites: number
  total_rated: number
  total_unrated: number
  avg_rating: number | null
}

export interface StatsResponse {
  overview: StatsOverview
  by_model_type: CountItem[]
  by_source_tool: CountItem[]
  by_model_name: CountItem[]
  by_sampler: CountItem[]
  by_lora: CountItem[]
  by_rating: RatingDistribution[]
  daily_counts: TimeSeriesItem[]
  daily_updates: TimeSeriesItem[]
}

export interface RatingAnalysisItem {
  name: string
  avg_rating: number
  count: number
  high_rated_count: number
}

export interface RatingAnalysisResponse {
  by_model: RatingAnalysisItem[]
  by_sampler: RatingAnalysisItem[]
  by_lora: RatingAnalysisItem[]
  by_steps: RatingAnalysisItem[]
  by_cfg: RatingAnalysisItem[]
  filtered_by_model: string | null
}

export interface ModelListResponse {
  models: string[]
}

export interface ModelRatingDistributionItem {
  model_name: string
  rating_0: number
  rating_1: number
  rating_2: number
  rating_3: number
  rating_4: number
  rating_5: number
  total: number
  avg_rating: number | null
}

export interface ModelRatingDistributionResponse {
  items: ModelRatingDistributionItem[]
}
