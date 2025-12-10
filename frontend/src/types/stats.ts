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
}
