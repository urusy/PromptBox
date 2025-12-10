import { Routes, Route, Navigate, useLocation } from 'react-router-dom'
import { useAuthStore } from '@/stores/authStore'
import MainLayout from '@/components/layout/MainLayout'
import LoginPage from '@/pages/LoginPage'
import GalleryPage from '@/pages/GalleryPage'
import DetailPage from '@/pages/DetailPage'
import TrashPage from '@/pages/TrashPage'
import DuplicatesPage from '@/pages/DuplicatesPage'
import SmartFoldersPage from '@/pages/SmartFoldersPage'
import StatsPage from '@/pages/StatsPage'

function PrivateRoute({ children }: { children: React.ReactNode }) {
  const isAuthenticated = useAuthStore((state) => state.isAuthenticated)
  const location = useLocation()

  if (!isAuthenticated) {
    // Preserve the current path and search params as returnUrl
    const returnUrl = location.pathname + location.search
    const loginUrl = returnUrl !== '/'
      ? `/login?returnUrl=${encodeURIComponent(returnUrl)}`
      : '/login'
    return <Navigate to={loginUrl} replace />
  }

  return <>{children}</>
}

function App() {
  return (
    <Routes>
      <Route path="/login" element={<LoginPage />} />
      <Route
        path="/"
        element={
          <PrivateRoute>
            <MainLayout />
          </PrivateRoute>
        }
      >
        <Route index element={<GalleryPage />} />
        <Route path="image/:id" element={<DetailPage />} />
        <Route path="trash" element={<TrashPage />} />
        <Route path="duplicates" element={<DuplicatesPage />} />
        <Route path="smart-folders" element={<SmartFoldersPage />} />
        <Route path="stats" element={<StatsPage />} />
      </Route>
    </Routes>
  )
}

export default App
