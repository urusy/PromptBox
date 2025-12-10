import client from './client'
import type { StatsResponse, RatingAnalysisResponse, ModelListResponse } from '@/types/stats'

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
}
