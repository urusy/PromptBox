import { create } from 'zustand'
import { persist } from 'zustand/middleware'
import { authApi } from '@/api/auth'

interface AuthState {
  username: string | null
  isAuthenticated: boolean
  login: (username: string, password: string) => Promise<void>
  logout: () => Promise<void>
  checkAuth: () => Promise<void>
}

export const useAuthStore = create<AuthState>()(
  persist(
    (set) => ({
      username: null,
      isAuthenticated: false,

      login: async (username: string, password: string) => {
        const response = await authApi.login({ username, password })
        set({ username: response.username, isAuthenticated: true })
      },

      logout: async () => {
        await authApi.logout()
        set({ username: null, isAuthenticated: false })
      },

      checkAuth: async () => {
        try {
          const user = await authApi.getMe()
          set({ username: user.username, isAuthenticated: true })
        } catch {
          set({ username: null, isAuthenticated: false })
        }
      },
    }),
    {
      name: 'auth-storage',
      partialize: (state) => ({ username: state.username, isAuthenticated: state.isAuthenticated }),
    }
  )
)
