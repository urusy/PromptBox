import client from './client'
import type {
  Showcase,
  ShowcaseCreate,
  ShowcaseDetail,
  ShowcaseImageAdd,
  ShowcaseImageCheck,
  ShowcaseImageCheckResult,
  ShowcaseImageRemove,
  ShowcaseImageReorder,
  ShowcaseUpdate,
} from '@/types/showcase'

export const showcasesApi = {
  list: async (): Promise<Showcase[]> => {
    const response = await client.get<Showcase[]>('/showcases')
    return response.data
  },

  get: async (id: string): Promise<ShowcaseDetail> => {
    const response = await client.get<ShowcaseDetail>(`/showcases/${id}`)
    return response.data
  },

  create: async (data: ShowcaseCreate): Promise<Showcase> => {
    const response = await client.post<Showcase>('/showcases', data)
    return response.data
  },

  update: async (id: string, data: ShowcaseUpdate): Promise<Showcase> => {
    const response = await client.put<Showcase>(`/showcases/${id}`, data)
    return response.data
  },

  delete: async (id: string): Promise<void> => {
    await client.delete(`/showcases/${id}`)
  },

  addImages: async (id: string, data: ShowcaseImageAdd): Promise<void> => {
    await client.post(`/showcases/${id}/images`, data)
  },

  removeImages: async (id: string, data: ShowcaseImageRemove): Promise<void> => {
    await client.delete(`/showcases/${id}/images`, { data })
  },

  reorderImages: async (id: string, data: ShowcaseImageReorder): Promise<void> => {
    await client.put(`/showcases/${id}/images/reorder`, data)
  },

  checkImages: async (data: ShowcaseImageCheck): Promise<ShowcaseImageCheckResult[]> => {
    const response = await client.post<ShowcaseImageCheckResult[]>('/showcases/check-images', data)
    return response.data
  },
}
