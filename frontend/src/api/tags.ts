import client from './client'

export const tagsApi = {
  /**
   * Get most recently used tags
   * @param limit Maximum number of tags to return (default: 10)
   */
  list: async (limit: number = 10): Promise<string[]> => {
    const { data } = await client.get<string[]>('/tags', {
      params: { limit },
    })
    return data
  },
}
