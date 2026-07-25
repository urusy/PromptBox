import { queryOptions } from '@tanstack/react-query'
import client from './client'
import type {
  Image,
  ImageListItem,
  ImageSearchParams,
  ImageUpdate,
  PaginatedResponse,
} from '@/types/image'
import { parseSearchParams, toApiParams } from '@/utils/searchParams'

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

/**
 * 詳細画面用クエリの共通定義。DetailPage 本体と各所のプリフェッチが
 * 同一のキー構造（['image', id, searchParams文字列]）を共有するために使う。
 * locationSearch には location.search をそのまま渡す（先頭の ? は有無どちらでも可）。
 */
export function imageDetailQueryOptions(id: string, locationSearch: string) {
  const searchParams = new URLSearchParams(locationSearch)
  return queryOptions({
    queryKey: ['image', id, searchParams.toString()] as const,
    queryFn: () => imagesApi.get(id, parseSearchParams(searchParams)),
  })
}
