import client from './client'
import type { SearchPreset, SearchPresetCreate, SearchPresetUpdate } from '@/types/searchPreset'

export const searchPresetsApi = {
  list: async (): Promise<SearchPreset[]> => {
    const response = await client.get<SearchPreset[]>('/search-presets')
    return response.data
  },

  create: async (data: SearchPresetCreate): Promise<SearchPreset> => {
    const response = await client.post<SearchPreset>('/search-presets', data)
    return response.data
  },

  update: async (id: string, data: SearchPresetUpdate): Promise<SearchPreset> => {
    const response = await client.put<SearchPreset>(`/search-presets/${id}`, data)
    return response.data
  },

  delete: async (id: string): Promise<void> => {
    await client.delete(`/search-presets/${id}`)
  },
}
