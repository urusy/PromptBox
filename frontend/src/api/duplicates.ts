import client from './client'

export interface DuplicatesInfo {
  count: number
  total_size_bytes: number
  files: string[]
}

export interface DeleteResult {
  deleted_count: number
  freed_bytes: number
}

export const duplicatesApi = {
  getInfo: async (): Promise<DuplicatesInfo> => {
    const response = await client.get<DuplicatesInfo>('/duplicates')
    return response.data
  },

  deleteAll: async (): Promise<DeleteResult> => {
    const response = await client.delete<DeleteResult>('/duplicates')
    return response.data
  },

  deleteFile: async (filename: string): Promise<{ deleted: string; freed_bytes: number }> => {
    const response = await client.delete(`/duplicates/${encodeURIComponent(filename)}`)
    return response.data
  },
}
