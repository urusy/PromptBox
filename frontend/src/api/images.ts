import client from './client'
import type {
  Image,
  ImageListItem,
  ImageSearchParams,
  ImageUpdate,
  PaginatedResponse,
} from '@/types/image'
import { toApiParams } from '@/utils/searchParams'

export const imagesApi = {
  list: async (params: ImageSearchParams = {}): Promise<PaginatedResponse<ImageListItem>> => {
    const response = await client.get<PaginatedResponse<ImageListItem>>('/images', { params })
    return response.data
  },

  get: async (id: string, searchParams?: ImageSearchParams): Promise<Image> => {
    const params = searchParams ? toApiParams(searchParams) : undefined
    const response = await client.get<Image>(`/images/${id}`, { params })
    return response.data
  },

  update: async (id: string, data: ImageUpdate): Promise<Image> => {
    const response = await client.patch<Image>(`/images/${id}`, data)
    return response.data
  },

  delete: async (id: string, permanent = false): Promise<void> => {
    await client.delete(`/images/${id}`, { params: { permanent } })
  },

  restore: async (id: string): Promise<void> => {
    await client.post(`/images/${id}/restore`)
  },
}
