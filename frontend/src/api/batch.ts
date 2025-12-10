import client from './client'

export interface BatchUpdateRequest {
  ids: string[]
  rating?: number
  is_favorite?: boolean
  needs_improvement?: boolean
  add_tags?: string[]
  remove_tags?: string[]
}

export interface BatchDeleteRequest {
  ids: string[]
  permanent?: boolean
}

export interface BatchRestoreRequest {
  ids: string[]
}

export const batchApi = {
  update: async (data: BatchUpdateRequest): Promise<void> => {
    await client.post('/bulk/update', data)
  },

  delete: async (data: BatchDeleteRequest): Promise<void> => {
    await client.post('/bulk/delete', data)
  },

  restore: async (data: BatchRestoreRequest): Promise<void> => {
    await client.post('/bulk/restore', data)
  },
}
