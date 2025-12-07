import client from './client'

export interface LoginRequest {
  username: string
  password: string
}

export interface LoginResponse {
  message: string
  username: string
}

export interface UserInfo {
  username: string
}

export const authApi = {
  login: async (data: LoginRequest): Promise<LoginResponse> => {
    const response = await client.post<LoginResponse>('/auth/login', data)
    return response.data
  },

  logout: async (): Promise<void> => {
    await client.post('/auth/logout')
  },

  getMe: async (): Promise<UserInfo> => {
    const response = await client.get<UserInfo>('/auth/me')
    return response.data
  },
}
