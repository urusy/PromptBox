import { useState } from 'react'
import { Outlet, Link, useNavigate, useLocation } from 'react-router-dom'
import {
  LogOut,
  Image,
  Trash2,
  Copy,
  FolderSearch,
  User,
  ChevronDown,
  BarChart3,
  Album,
  Box,
  Layers,
  Sparkles,
  Menu,
  X,
} from 'lucide-react'
import { useAuthStore } from '@/stores/authStore'

export default function MainLayout() {
  const { username, logout } = useAuthStore()
  const navigate = useNavigate()
  const location = useLocation()
  const [showUserMenu, setShowUserMenu] = useState(false)
  const [showMobileMenu, setShowMobileMenu] = useState(false)

  const handleLogout = async () => {
    setShowUserMenu(false)
    setShowMobileMenu(false)
    await logout()
    navigate('/login')
  }

  const navItems = [
    { to: '/', icon: Image, label: 'Gallery', color: '' },
    { to: '/swipe', icon: Sparkles, label: 'Quick Rate', color: 'text-yellow-400' },
    { to: '/trash', icon: Trash2, label: 'Trash', color: '' },
    { to: '/duplicates', icon: Copy, label: 'Duplicates', color: '' },
    { to: '/smart-folders', icon: FolderSearch, label: 'Smart Folders', color: '' },
    { to: '/stats', icon: BarChart3, label: 'Stats', color: '' },
    { to: '/models', icon: Box, label: 'Models', color: 'text-blue-400' },
    { to: '/loras', icon: Layers, label: 'LoRAs', color: 'text-orange-400' },
  ]

  const isActive = (path: string) => location.pathname === path

  return (
    <div className="min-h-screen bg-gray-900 text-white overflow-x-hidden">
      <header
        className="sticky top-0 z-50 bg-gray-800 border-b border-gray-700"
        style={{
          paddingTop: 'env(safe-area-inset-top)',
        }}
      >
        <div
          className="mx-auto"
          style={{
            paddingLeft: 'max(0.5rem, env(safe-area-inset-left))',
            paddingRight: 'max(0.5rem, env(safe-area-inset-right))',
          }}
        >
          <div className="flex items-center justify-between h-14 sm:h-16">
            {/* Logo */}
            <div className="flex items-center gap-2 lg:gap-4 shrink-0">
              <Link to="/" className="text-lg sm:text-xl font-bold shrink-0">
                Prompt Box
              </Link>
              {/* Showcase - visible on desktop, icon only until xl */}
              <Link
                to="/showcases"
                className="hidden md:flex items-center gap-1.5 px-2 xl:px-3 py-1.5 rounded-md bg-gray-700 hover:bg-gray-600 transition-colors"
                title="Showcases"
              >
                <Album size={16} />
                <span className="hidden xl:inline text-sm">Showcase</span>
              </Link>
            </div>

            {/* Desktop Navigation - hidden on mobile */}
            <nav className="hidden md:flex items-center gap-0.5 xl:gap-2">
              {navItems.map((item) => (
                <Link
                  key={item.to}
                  to={item.to}
                  className={`flex items-center gap-1.5 px-2 xl:px-3 py-1.5 xl:py-2 rounded-md transition-colors ${
                    isActive(item.to) ? 'bg-gray-700' : 'hover:bg-gray-700'
                  }`}
                  title={item.label}
                >
                  <item.icon size={16} className={`xl:w-[18px] xl:h-[18px] ${item.color}`} />
                  <span className="hidden xl:inline text-sm">{item.label}</span>
                </Link>
              ))}

              {/* Divider */}
              <div className="w-px h-5 lg:h-6 bg-gray-600 mx-1 lg:mx-2" />

              {/* User menu */}
              <div className="relative">
                <button
                  onClick={() => setShowUserMenu(!showUserMenu)}
                  className="flex items-center gap-1 xl:gap-1.5 px-2 xl:px-3 py-1.5 lg:py-2 rounded-md hover:bg-gray-700 transition-colors"
                  title={username || undefined}
                >
                  <User size={16} className="lg:w-[18px] lg:h-[18px]" />
                  <span className="hidden xl:inline text-sm text-gray-300 max-w-[80px] truncate">{username}</span>
                  <ChevronDown
                    size={14}
                    className={`hidden xl:inline text-gray-400 transition-transform ${showUserMenu ? 'rotate-180' : ''}`}
                  />
                </button>

                {showUserMenu && (
                  <>
                    <div className="fixed inset-0 z-10" onClick={() => setShowUserMenu(false)} />
                    <div className="absolute right-0 top-full mt-1 w-48 bg-gray-700 border border-gray-600 rounded-lg shadow-xl z-20">
                      <button
                        onClick={handleLogout}
                        className="flex items-center gap-2 w-full px-3 py-2 text-sm text-gray-300 hover:bg-gray-600 hover:text-white transition-colors rounded-lg"
                      >
                        <LogOut size={16} />
                        Logout
                      </button>
                    </div>
                  </>
                )}
              </div>
            </nav>

            {/* Mobile menu button */}
            <button
              onClick={() => setShowMobileMenu(!showMobileMenu)}
              className="md:hidden p-2 rounded-md hover:bg-gray-700 transition-colors"
              aria-label="Toggle menu"
            >
              {showMobileMenu ? <X size={24} /> : <Menu size={24} />}
            </button>
          </div>

          {/* Mobile Navigation Menu */}
          {showMobileMenu && (
            <div className="md:hidden border-t border-gray-700 py-2">
              <nav className="flex flex-col">
                {/* Showcase */}
                <Link
                  to="/showcases"
                  onClick={() => setShowMobileMenu(false)}
                  className={`flex items-center gap-3 px-4 py-3 transition-colors ${
                    isActive('/showcases') ? 'bg-gray-700' : 'hover:bg-gray-700'
                  }`}
                >
                  <Album size={20} />
                  <span>Showcase</span>
                </Link>

                {/* Nav items */}
                {navItems.map((item) => (
                  <Link
                    key={item.to}
                    to={item.to}
                    onClick={() => setShowMobileMenu(false)}
                    className={`flex items-center gap-3 px-4 py-3 transition-colors ${
                      isActive(item.to) ? 'bg-gray-700' : 'hover:bg-gray-700'
                    }`}
                  >
                    <item.icon size={20} className={item.color} />
                    <span>{item.label}</span>
                  </Link>
                ))}

                {/* Divider */}
                <div className="h-px bg-gray-700 my-2" />

                {/* User info and logout */}
                <div className="px-4 py-2 text-sm text-gray-400">
                  <User size={16} className="inline mr-2" />
                  {username}
                </div>
                <button
                  onClick={handleLogout}
                  className="flex items-center gap-3 px-4 py-3 text-left hover:bg-gray-700 transition-colors text-red-400"
                >
                  <LogOut size={20} />
                  <span>Logout</span>
                </button>
              </nav>
            </div>
          )}
        </div>
      </header>
      <main
        className="px-2 sm:px-4 lg:px-8 py-4 sm:py-8"
        style={{
          paddingLeft: 'max(0.5rem, env(safe-area-inset-left))',
          paddingRight: 'max(0.5rem, env(safe-area-inset-right))',
          paddingBottom: 'env(safe-area-inset-bottom)',
        }}
      >
        <Outlet />
      </main>
    </div>
  )
}
