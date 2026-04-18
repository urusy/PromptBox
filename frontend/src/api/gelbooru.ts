import client from './client'
import type { GelbooruTagSearchResponse } from '@/types/gelbooru'

export const gelbooruApi = {
  searchTags: async (q: string, limit: number = 30): Promise<GelbooruTagSearchResponse> => {
    const response = await client.get<GelbooruTagSearchResponse>('/gelbooru/tags', {
      params: { q, limit },
    })
    return response.data
  },
}
