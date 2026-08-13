import { queryOptions } from '@tanstack/react-query'
import client from './client'
import type { GridMembersResponse } from '@/types/grid'

export const gridsApi = {
  /**
   * グリッドを構成する画像を取得する。バックエンド側の推定結果なので、
   * confidence を必ず併せて表示すること。
   */
  members: async (id: string, windowHours?: number): Promise<GridMembersResponse> => {
    const response = await client.get<GridMembersResponse>(`/images/${id}/grid-members`, {
      params: windowHours ? { window_hours: windowHours } : undefined,
    })
    return response.data
  },
}

export function gridMembersQueryOptions(id: string, windowHours?: number) {
  return queryOptions({
    queryKey: ['grid-members', id, windowHours ?? null] as const,
    queryFn: () => gridsApi.members(id, windowHours),
    // グリッドでない画像には 400 が返る。再試行しても変わらない。
    retry: false,
  })
}
