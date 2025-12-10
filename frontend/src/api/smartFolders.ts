import client from './client'
import type { SmartFolder, SmartFolderCreate, SmartFolderUpdate } from '@/types/smartFolder'

export const smartFoldersApi = {
  list: async (): Promise<SmartFolder[]> => {
    const response = await client.get<SmartFolder[]>('/smart-folders')
    return response.data
  },

  get: async (id: string): Promise<SmartFolder> => {
    const response = await client.get<SmartFolder>(`/smart-folders/${id}`)
    return response.data
  },

  create: async (data: SmartFolderCreate): Promise<SmartFolder> => {
    const response = await client.post<SmartFolder>('/smart-folders', data)
    return response.data
  },

  update: async (id: string, data: SmartFolderUpdate): Promise<SmartFolder> => {
    const response = await client.put<SmartFolder>(`/smart-folders/${id}`, data)
    return response.data
  },

  delete: async (id: string): Promise<void> => {
    await client.delete(`/smart-folders/${id}`)
  },
}
