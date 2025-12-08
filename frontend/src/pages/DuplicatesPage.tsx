import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { Trash2, AlertTriangle, RefreshCw } from 'lucide-react'
import toast from 'react-hot-toast'
import { duplicatesApi } from '@/api/duplicates'

export default function DuplicatesPage() {
  const queryClient = useQueryClient()

  const { data, isLoading, error, refetch } = useQuery({
    queryKey: ['duplicates'],
    queryFn: duplicatesApi.getInfo,
  })

  const deleteAllMutation = useMutation({
    mutationFn: duplicatesApi.deleteAll,
    onSuccess: (result) => {
      toast.success(`Deleted ${result.deleted_count} files (${formatFileSize(result.freed_bytes)} freed)`)
      queryClient.invalidateQueries({ queryKey: ['duplicates'] })
    },
    onError: () => {
      toast.error('Failed to delete files')
    },
  })

  const deleteFileMutation = useMutation({
    mutationFn: duplicatesApi.deleteFile,
    onSuccess: (result) => {
      toast.success(`Deleted ${result.deleted}`)
      queryClient.invalidateQueries({ queryKey: ['duplicates'] })
    },
    onError: () => {
      toast.error('Failed to delete file')
    },
  })

  const formatFileSize = (bytes: number) => {
    if (bytes < 1024) return `${bytes} B`
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
  }

  const handleDeleteAll = () => {
    if (window.confirm(`Are you sure you want to delete all ${data?.count} duplicate files? This cannot be undone.`)) {
      deleteAllMutation.mutate()
    }
  }

  const handleDeleteFile = (filename: string) => {
    if (window.confirm(`Delete ${filename}?`)) {
      deleteFileMutation.mutate(filename)
    }
  }

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin rounded-full h-12 w-12 border-t-2 border-b-2 border-blue-500"></div>
      </div>
    )
  }

  if (error) {
    return (
      <div className="text-center text-red-500 py-8">
        Failed to load duplicates info
      </div>
    )
  }

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-2xl font-bold">Duplicate Files</h1>
        <button
          onClick={() => refetch()}
          className="flex items-center gap-2 px-3 py-2 rounded-md bg-gray-700 hover:bg-gray-600 transition-colors"
        >
          <RefreshCw size={18} />
          <span>Refresh</span>
        </button>
      </div>

      {/* Summary Card */}
      <div className="bg-gray-800 rounded-lg p-6 mb-6">
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          <div>
            <p className="text-gray-400 text-sm">Total Files</p>
            <p className="text-3xl font-bold">{data?.count || 0}</p>
          </div>
          <div>
            <p className="text-gray-400 text-sm">Total Size</p>
            <p className="text-3xl font-bold">{formatFileSize(data?.total_size_bytes || 0)}</p>
          </div>
          <div className="flex items-end">
            {data && data.count > 0 && (
              <button
                onClick={handleDeleteAll}
                disabled={deleteAllMutation.isPending}
                className="flex items-center gap-2 px-4 py-2 rounded-md bg-red-600 hover:bg-red-500 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
              >
                <Trash2 size={18} />
                <span>{deleteAllMutation.isPending ? 'Deleting...' : 'Delete All'}</span>
              </button>
            )}
          </div>
        </div>
      </div>

      {/* Empty State */}
      {(!data || data.count === 0) && (
        <div className="text-center py-12 text-gray-500">
          <AlertTriangle size={48} className="mx-auto mb-4 opacity-50" />
          <p>No duplicate files found</p>
          <p className="text-sm mt-2">Duplicate files will appear here when importing images that already exist in the gallery.</p>
        </div>
      )}

      {/* File List */}
      {data && data.count > 0 && (
        <div className="bg-gray-800 rounded-lg overflow-hidden">
          <table className="w-full">
            <thead>
              <tr className="border-b border-gray-700">
                <th className="text-left px-4 py-3 text-gray-400 font-medium">Filename</th>
                <th className="text-right px-4 py-3 text-gray-400 font-medium w-24">Action</th>
              </tr>
            </thead>
            <tbody>
              {data.files.map((filename) => (
                <tr key={filename} className="border-b border-gray-700 last:border-0 hover:bg-gray-750">
                  <td className="px-4 py-3 font-mono text-sm">{filename}</td>
                  <td className="px-4 py-3 text-right">
                    <button
                      onClick={() => handleDeleteFile(filename)}
                      disabled={deleteFileMutation.isPending}
                      className="p-2 rounded hover:bg-gray-700 text-gray-400 hover:text-red-500 transition-colors"
                      title="Delete"
                    >
                      <Trash2 size={18} />
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  )
}
