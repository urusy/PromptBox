import { lazy, Suspense } from 'react'
import { Routes, Route, Navigate, useLocation } from 'react-router-dom'
import { useAuthStore } from '@/stores/authStore'
import ErrorBoundary from '@/components/common/ErrorBoundary'
import PageLoader from '@/components/common/PageLoader'
import MainLayout from '@/components/layout/MainLayout'
import LoginPage from '@/pages/LoginPage'
import GalleryPage from '@/pages/GalleryPage'
import DetailPage from '@/pages/DetailPage'

// Lazy load less frequently used pages
const TrashPage = lazy(() => import('@/pages/TrashPage'))
const DuplicatesPage = lazy(() => import('@/pages/DuplicatesPage'))
const SmartFoldersPage = lazy(() => import('@/pages/SmartFoldersPage'))
const ShowcasesPage = lazy(() => import('@/pages/ShowcasesPage'))
const ShowcaseDetailPage = lazy(() => import('@/pages/ShowcaseDetailPage'))
const StatsPage = lazy(() => import('@/pages/StatsPage'))
const ModelsPage = lazy(() => import('@/pages/ModelsPage'))
const ModelDetailPage = lazy(() => import('@/pages/ModelDetailPage'))
const LorasPage = lazy(() => import('@/pages/LorasPage'))
const LoraDetailPage = lazy(() => import('@/pages/LoraDetailPage'))
const SwipePage = lazy(() => import('@/pages/SwipePage'))

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
    <ErrorBoundary>
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
          <Route
            path="trash"
            element={
              <Suspense fallback={<PageLoader />}>
                <TrashPage />
              </Suspense>
            }
          />
          <Route
            path="duplicates"
            element={
              <Suspense fallback={<PageLoader />}>
                <DuplicatesPage />
              </Suspense>
            }
          />
          <Route
            path="smart-folders"
            element={
              <Suspense fallback={<PageLoader />}>
                <SmartFoldersPage />
              </Suspense>
            }
          />
          <Route
            path="showcases"
            element={
              <Suspense fallback={<PageLoader />}>
                <ShowcasesPage />
              </Suspense>
            }
          />
          <Route
            path="showcase/:id"
            element={
              <Suspense fallback={<PageLoader />}>
                <ShowcaseDetailPage />
              </Suspense>
            }
          />
          <Route
            path="stats"
            element={
              <Suspense fallback={<PageLoader />}>
                <StatsPage />
              </Suspense>
            }
          />
          <Route
            path="models"
            element={
              <Suspense fallback={<PageLoader />}>
                <ModelsPage />
              </Suspense>
            }
          />
          <Route
            path="models/:name"
            element={
              <Suspense fallback={<PageLoader />}>
                <ModelDetailPage />
              </Suspense>
            }
          />
          <Route
            path="loras"
            element={
              <Suspense fallback={<PageLoader />}>
                <LorasPage />
              </Suspense>
            }
          />
          <Route
            path="loras/:name"
            element={
              <Suspense fallback={<PageLoader />}>
                <LoraDetailPage />
              </Suspense>
            }
          />
          <Route
            path="swipe"
            element={
              <Suspense fallback={<PageLoader />}>
                <SwipePage />
              </Suspense>
            }
          />
        </Route>
      </Routes>
    </ErrorBoundary>
  )
}

export default App
