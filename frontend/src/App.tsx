import { Routes, Route, Navigate } from 'react-router-dom'
import { useAuthStore } from '@/stores/authStore'
import MainLayout from '@/components/layout/MainLayout'
import LoginPage from '@/pages/LoginPage'
import GalleryPage from '@/pages/GalleryPage'
import DetailPage from '@/pages/DetailPage'
import TrashPage from '@/pages/TrashPage'
import DuplicatesPage from '@/pages/DuplicatesPage'

function PrivateRoute({ children }: { children: React.ReactNode }) {
  const isAuthenticated = useAuthStore((state) => state.isAuthenticated)

  if (!isAuthenticated) {
    return <Navigate to="/login" replace />
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
      </Route>
    </Routes>
  )
}

export default App
