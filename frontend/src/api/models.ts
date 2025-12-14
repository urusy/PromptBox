import client from './client'
import type {
  ModelListResponse,
  ModelDetail,
  LoraListResponse,
  LoraDetail,
  CivitaiInfoResponse,
  ModelSearchParams,
  LoraSearchParams,
} from '@/types/model'

export const modelsApi = {
  list: async (params: ModelSearchParams = {}): Promise<ModelListResponse> => {
    const response = await client.get<ModelListResponse>('/models', {
      params: {
        q: params.q,
        model_type: params.model_type,
        min_count: params.min_count,
        min_rating: params.min_rating,
        sort_by: params.sort_by,
        sort_order: params.sort_order,
        limit: params.limit,
        offset: params.offset,
      },
    })
    return response.data
  },

  getDetail: async (modelName: string): Promise<ModelDetail> => {
    const response = await client.get<ModelDetail>(
      `/models/${encodeURIComponent(modelName)}/detail`
    )
    return response.data
  },

  getCivitaiInfo: async (modelName: string): Promise<CivitaiInfoResponse> => {
    const response = await client.get<CivitaiInfoResponse>(
      `/models/${encodeURIComponent(modelName)}/civitai`
    )
    return response.data
  },
}

export const lorasApi = {
  list: async (params: LoraSearchParams = {}): Promise<LoraListResponse> => {
    const response = await client.get<LoraListResponse>('/loras', {
      params: {
        q: params.q,
        min_count: params.min_count,
        min_rating: params.min_rating,
        sort_by: params.sort_by,
        sort_order: params.sort_order,
        limit: params.limit,
        offset: params.offset,
      },
    })
    return response.data
  },

  getDetail: async (loraName: string): Promise<LoraDetail> => {
    const response = await client.get<LoraDetail>(`/loras/${encodeURIComponent(loraName)}/detail`)
    return response.data
  },

  getCivitaiInfo: async (loraName: string): Promise<CivitaiInfoResponse> => {
    const response = await client.get<CivitaiInfoResponse>(
      `/loras/${encodeURIComponent(loraName)}/civitai`
    )
    return response.data
  },
}
