import client from './client'
import type { StatsResponse, RatingAnalysisResponse, ModelListResponse, LoraListResponse, ModelRatingDistributionResponse } from '@/types/stats'

export const statsApi = {
  get: async (days: number = 30): Promise<StatsResponse> => {
    const response = await client.get<StatsResponse>('/stats', {
      params: { days },
    })
    return response.data
  },

  getRatingAnalysis: async (minCount: number = 5, modelName?: string): Promise<RatingAnalysisResponse> => {
    const response = await client.get<RatingAnalysisResponse>('/stats/rating-analysis', {
      params: { min_count: minCount, model_name: modelName },
    })
    return response.data
  },

  getModelsForAnalysis: async (minCount: number = 5): Promise<ModelListResponse> => {
    const response = await client.get<ModelListResponse>('/stats/models-for-analysis', {
      params: { min_count: minCount },
    })
    return response.data
  },

  getLorasForFilter: async (minCount: number = 1): Promise<LoraListResponse> => {
    const response = await client.get<LoraListResponse>('/stats/loras-for-filter', {
      params: { min_count: minCount },
    })
    return response.data
  },

  getModelRatingDistribution: async (minCount: number = 10, limit: number = 15): Promise<ModelRatingDistributionResponse> => {
    const response = await client.get<ModelRatingDistributionResponse>('/stats/model-rating-distribution', {
      params: { min_count: minCount, limit },
    })
    return response.data
  },
}
