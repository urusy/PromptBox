import client from './client'

export const tagsApi = {
  /**
   * Get tags with optional search filtering
   * @param q Search query to filter tags (optional)
   * @param limit Maximum number of tags to return (default: 10)
   */
  list: async (q?: string, limit: number = 10): Promise<string[]> => {
    const { data } = await client.get<string[]>('/tags', {
      params: { q: q || undefined, limit },
    })
    return data
  },
}
