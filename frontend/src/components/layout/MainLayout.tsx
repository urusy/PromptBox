import { Outlet, Link, useNavigate } from 'react-router-dom'
import { LogOut, Image, Trash2, Copy } from 'lucide-react'
import { useAuthStore } from '@/stores/authStore'

export default function MainLayout() {
  const { username, logout } = useAuthStore()
  const navigate = useNavigate()

  const handleLogout = async () => {
    await logout()
    navigate('/login')
  }

  return (
    <div className="min-h-screen bg-gray-900 text-white">
      <header className="sticky top-0 z-50 bg-gray-800 border-b border-gray-700">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex items-center justify-between h-16">
            <div className="flex items-center gap-8">
              <Link to="/" className="text-xl font-bold">
                ComfyUI Gallery
              </Link>
              <nav className="flex items-center gap-4">
                <Link
                  to="/"
                  className="flex items-center gap-2 px-3 py-2 rounded-md hover:bg-gray-700 transition-colors"
                >
                  <Image size={18} />
                  <span>Gallery</span>
                </Link>
                <Link
                  to="/trash"
                  className="flex items-center gap-2 px-3 py-2 rounded-md hover:bg-gray-700 transition-colors"
                >
                  <Trash2 size={18} />
                  <span>Trash</span>
                </Link>
                <Link
                  to="/duplicates"
                  className="flex items-center gap-2 px-3 py-2 rounded-md hover:bg-gray-700 transition-colors"
                >
                  <Copy size={18} />
                  <span>Duplicates</span>
                </Link>
              </nav>
            </div>
            <div className="flex items-center gap-4">
              <span className="text-sm text-gray-400">{username}</span>
              <button
                onClick={handleLogout}
                className="flex items-center gap-2 px-3 py-2 rounded-md hover:bg-gray-700 transition-colors"
              >
                <LogOut size={18} />
                <span>Logout</span>
              </button>
            </div>
          </div>
        </div>
      </header>
      <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <Outlet />
      </main>
    </div>
  )
}
