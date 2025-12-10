import client from './client'
import type { StatsResponse } from '@/types/stats'

export const statsApi = {
  get: async (days: number = 30): Promise<StatsResponse> => {
    const response = await client.get<StatsResponse>('/stats', {
      params: { days },
    })
    return response.data
  },
}
