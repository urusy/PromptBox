import { useState } from 'react'
import { Outlet, Link, useNavigate } from 'react-router-dom'
import { LogOut, Image, Trash2, Copy, FolderSearch, User, ChevronDown, BarChart3 } from 'lucide-react'
import { useAuthStore } from '@/stores/authStore'

export default function MainLayout() {
  const { username, logout } = useAuthStore()
  const navigate = useNavigate()
  const [showUserMenu, setShowUserMenu] = useState(false)

  const handleLogout = async () => {
    setShowUserMenu(false)
    await logout()
    navigate('/login')
  }

  return (
    <div className="min-h-screen bg-gray-900 text-white overflow-x-hidden">
      <header className="sticky top-0 z-50 bg-gray-800 border-b border-gray-700">
        <div className="mx-auto px-2 sm:px-4 lg:px-8 max-w-[1920px] 4xl:max-w-none">
          <div className="flex items-center justify-between h-14 sm:h-16">
            {/* Logo */}
            <Link to="/" className="text-lg sm:text-xl font-bold shrink-0">
              Prompt Box
            </Link>

            {/* Navigation - Icons only on mobile */}
            <nav className="flex items-center gap-1 sm:gap-2">
              <Link
                to="/"
                className="flex items-center gap-2 p-2 sm:px-3 sm:py-2 rounded-md hover:bg-gray-700 transition-colors"
                title="Gallery"
              >
                <Image size={18} />
                <span className="hidden sm:inline text-sm">Gallery</span>
              </Link>
              <Link
                to="/trash"
                className="flex items-center gap-2 p-2 sm:px-3 sm:py-2 rounded-md hover:bg-gray-700 transition-colors"
                title="Trash"
              >
                <Trash2 size={18} />
                <span className="hidden sm:inline text-sm">Trash</span>
              </Link>
              <Link
                to="/duplicates"
                className="flex items-center gap-2 p-2 sm:px-3 sm:py-2 rounded-md hover:bg-gray-700 transition-colors"
                title="Duplicates"
              >
                <Copy size={18} />
                <span className="hidden md:inline text-sm">Duplicates</span>
              </Link>
              <Link
                to="/smart-folders"
                className="flex items-center gap-2 p-2 sm:px-3 sm:py-2 rounded-md hover:bg-gray-700 transition-colors"
                title="Smart Folders"
              >
                <FolderSearch size={18} />
                <span className="hidden md:inline text-sm">Smart</span>
              </Link>
              <Link
                to="/stats"
                className="flex items-center gap-2 p-2 sm:px-3 sm:py-2 rounded-md hover:bg-gray-700 transition-colors"
                title="Statistics"
              >
                <BarChart3 size={18} />
                <span className="hidden md:inline text-sm">Stats</span>
              </Link>

              {/* Divider */}
              <div className="w-px h-6 bg-gray-600 mx-1 sm:mx-2" />

              {/* User menu */}
              <div className="relative">
                <button
                  onClick={() => setShowUserMenu(!showUserMenu)}
                  className="flex items-center gap-1.5 p-2 sm:px-3 sm:py-2 rounded-md hover:bg-gray-700 transition-colors"
                  title={username || undefined}
                >
                  <User size={18} />
                  <span className="hidden sm:inline text-sm text-gray-300">{username}</span>
                  <ChevronDown size={14} className={`hidden sm:block text-gray-400 transition-transform ${showUserMenu ? 'rotate-180' : ''}`} />
                </button>

                {showUserMenu && (
                  <>
                    <div
                      className="fixed inset-0 z-10"
                      onClick={() => setShowUserMenu(false)}
                    />
                    <div className="absolute right-0 top-full mt-1 w-48 bg-gray-700 border border-gray-600 rounded-lg shadow-xl z-20">
                      <div className="px-3 py-2 border-b border-gray-600 sm:hidden">
                        <span className="text-sm text-gray-300">{username}</span>
                      </div>
                      <button
                        onClick={handleLogout}
                        className="flex items-center gap-2 w-full px-3 py-2 text-sm text-gray-300 hover:bg-gray-600 hover:text-white transition-colors rounded-b-lg"
                      >
                        <LogOut size={16} />
                        Logout
                      </button>
                    </div>
                  </>
                )}
              </div>
            </nav>
          </div>
        </div>
      </header>
      <main className="mx-auto px-2 sm:px-4 lg:px-8 py-4 sm:py-8 max-w-[1920px] 4xl:max-w-none">
        <Outlet />
      </main>
    </div>
  )
}
