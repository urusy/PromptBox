import { useState, useEffect, useCallback } from 'react'
import { useParams, useNavigate, useSearchParams, useLocation } from 'react-router-dom'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { ArrowLeft, Heart, Trash2, Copy, AlertTriangle, Download, X, ChevronLeft, ChevronRight } from 'lucide-react'
import toast from 'react-hot-toast'
import { imagesApi } from '@/api/images'
import type { ImageUpdate } from '@/types/image'
import { parseSearchParams } from '@/utils/searchParams'
import StarRating from '@/components/common/StarRating'
import TagEditor from '@/components/detail/TagEditor'
import MemoEditor from '@/components/detail/MemoEditor'

export default function DetailPage() {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const location = useLocation()
  const [urlSearchParams] = useSearchParams()
  const queryClient = useQueryClient()
  const [isLightboxOpen, setIsLightboxOpen] = useState(false)

  // Parse search params from URL to pass to API
  const searchParams = parseSearchParams(urlSearchParams)

  const { data: image, isLoading, error } = useQuery({
    queryKey: ['image', id, urlSearchParams.toString()],
    queryFn: () => imagesApi.get(id!, searchParams),
    enabled: !!id,
  })

  // Navigate to prev/next image while preserving search params
  const navigateToImage = useCallback((imageId: string) => {
    navigate(`/images/${imageId}${location.search}`)
  }, [navigate, location.search])

  // Keyboard navigation handler
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Ignore if user is typing in an input field
      const target = e.target as HTMLElement
      if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable) {
        return
      }

      // Handle Escape key for lightbox
      if (e.key === 'Escape' && isLightboxOpen) {
        setIsLightboxOpen(false)
        return
      }

      if (!image) return

      // Left arrow - go to previous image
      if (e.key === 'ArrowLeft' && image.prev_id) {
        navigateToImage(image.prev_id)
      }
      // Right arrow - go to next image
      else if (e.key === 'ArrowRight' && image.next_id) {
        navigateToImage(image.next_id)
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [image, navigateToImage, isLightboxOpen])

  const updateMutation = useMutation({
    mutationFn: (data: ImageUpdate) => imagesApi.update(id!, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['image', id] })
      queryClient.invalidateQueries({ queryKey: ['images'] })
    },
  })

  const deleteMutation = useMutation({
    mutationFn: () => imagesApi.delete(id!),
    onSuccess: () => {
      toast.success('Image moved to trash')
      navigate('/')
    },
  })

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text)
    toast.success('Copied to clipboard')
  }

  const handleDownload = () => {
    if (!image) return
    const link = document.createElement('a')
    link.href = `/storage/${image.storage_path}`
    link.download = image.original_filename
    link.click()
  }

  const formatFileSize = (bytes: number) => {
    if (bytes < 1024) return `${bytes} B`
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
  }

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin rounded-full h-12 w-12 border-t-2 border-b-2 border-blue-500"></div>
      </div>
    )
  }

  if (error || !image) {
    return (
      <div className="text-center text-red-500 py-8">
        Image not found
      </div>
    )
  }

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <button
          onClick={() => navigate(-1)}
          className="flex items-center gap-2 text-gray-400 hover:text-white"
        >
          <ArrowLeft size={20} />
          <span>Back</span>
        </button>

        {/* Navigation buttons */}
        <div className="flex items-center gap-2">
          <button
            onClick={() => image?.prev_id && navigateToImage(image.prev_id)}
            disabled={!image?.prev_id}
            className={`flex items-center gap-1 px-3 py-1.5 rounded-lg text-sm transition-colors ${
              image?.prev_id
                ? 'bg-gray-800 text-gray-300 hover:text-white hover:bg-gray-700'
                : 'bg-gray-800/50 text-gray-600 cursor-not-allowed'
            }`}
            title="Previous image (←)"
          >
            <ChevronLeft size={18} />
            <span className="hidden sm:inline">Prev</span>
          </button>
          <button
            onClick={() => image?.next_id && navigateToImage(image.next_id)}
            disabled={!image?.next_id}
            className={`flex items-center gap-1 px-3 py-1.5 rounded-lg text-sm transition-colors ${
              image?.next_id
                ? 'bg-gray-800 text-gray-300 hover:text-white hover:bg-gray-700'
                : 'bg-gray-800/50 text-gray-600 cursor-not-allowed'
            }`}
            title="Next image (→)"
          >
            <span className="hidden sm:inline">Next</span>
            <ChevronRight size={18} />
          </button>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
        <div>
          <img
            src={`/storage/${image.storage_path}`}
            alt={image.model_name || 'Generated image'}
            className="w-full rounded-lg cursor-pointer hover:opacity-90 transition-opacity"
            onClick={() => setIsLightboxOpen(true)}
            title="Click to enlarge"
          />
        </div>

        <div className="space-y-6">
          {/* Actions */}
          <div className="flex items-center gap-4 flex-wrap">
            <StarRating
              rating={image.rating}
              onChange={(rating) => updateMutation.mutate({ rating })}
              size={28}
            />
            <button
              onClick={() => updateMutation.mutate({ is_favorite: !image.is_favorite })}
              className="p-2 rounded-lg hover:bg-gray-800 transition-colors"
              title={image.is_favorite ? 'Remove from favorites' : 'Add to favorites'}
            >
              <Heart
                size={24}
                className={image.is_favorite ? 'text-red-500' : 'text-gray-600'}
                fill={image.is_favorite ? 'currentColor' : 'none'}
              />
            </button>
            <button
              onClick={() => updateMutation.mutate({ needs_improvement: !image.needs_improvement })}
              className="p-2 rounded-lg hover:bg-gray-800 transition-colors"
              title={image.needs_improvement ? 'Remove improvement flag' : 'Mark for improvement'}
            >
              <AlertTriangle
                size={24}
                className={image.needs_improvement ? 'text-yellow-500' : 'text-gray-600'}
                fill={image.needs_improvement ? 'currentColor' : 'none'}
              />
            </button>
            <button
              onClick={handleDownload}
              className="p-2 rounded-lg hover:bg-gray-800 transition-colors text-gray-400 hover:text-white"
              title="Download original"
            >
              <Download size={24} />
            </button>
            <button
              onClick={() => deleteMutation.mutate()}
              className="p-2 rounded-lg hover:bg-gray-800 transition-colors text-gray-600 hover:text-red-500"
              title="Move to trash"
            >
              <Trash2 size={24} />
            </button>
          </div>

          {/* Basic Info */}
          <div className="space-y-3">
            <InfoRow label="Model" value={image.model_name} />
            <InfoRow label="Model Type" value={image.model_type?.toUpperCase()} />
            <InfoRow label="Source" value={image.source_tool.toUpperCase()} />
            <InfoRow label="Size" value={`${image.width} × ${image.height}`} />
            <InfoRow label="File Size" value={formatFileSize(image.file_size_bytes)} />
            <InfoRow label="Sampler" value={image.sampler_name} />
            <InfoRow label="Scheduler" value={image.scheduler} />
            <InfoRow label="Steps" value={image.steps?.toString()} />
            <InfoRow label="CFG Scale" value={image.cfg_scale?.toString()} />
            <InfoRow label="Seed" value={image.seed?.toString()} />
            {/* Hires/Upscale Info */}
            <InfoRow label="Hires Upscaler" value={image.model_params?.hires_upscaler as string | undefined} />
            <InfoRow label="Hires Scale" value={(image.model_params?.hires_upscale as number | undefined)?.toString()} />
            <InfoRow label="Hires Steps" value={(image.model_params?.hires_steps as number | undefined)?.toString()} />
            <InfoRow label="Denoising" value={(image.model_params?.denoising_strength as number | undefined)?.toString()} />
          </div>

          {/* XYZ Grid Info */}
          {Boolean(image.model_params?.is_xyz_grid) && (
            <div>
              <h3 className="text-lg font-medium text-white mb-2">XYZ Grid</h3>
              <div className="bg-gray-800 p-3 rounded-lg space-y-2">
                {Boolean(image.model_params?.xyz_x_type) && (
                  <div className="flex items-start gap-2">
                    <span className="text-gray-400 font-medium min-w-16">X Axis:</span>
                    <div>
                      <span className="text-blue-400">{String(image.model_params?.xyz_x_type)}</span>
                      {Boolean(image.model_params?.xyz_x_values) && (
                        <p className="text-sm text-gray-300 mt-1 font-mono">{String(image.model_params?.xyz_x_values)}</p>
                      )}
                    </div>
                  </div>
                )}
                {Boolean(image.model_params?.xyz_y_type) && (
                  <div className="flex items-start gap-2">
                    <span className="text-gray-400 font-medium min-w-16">Y Axis:</span>
                    <div>
                      <span className="text-green-400">{String(image.model_params?.xyz_y_type)}</span>
                      {Boolean(image.model_params?.xyz_y_values) && (
                        <p className="text-sm text-gray-300 mt-1 font-mono">{String(image.model_params?.xyz_y_values)}</p>
                      )}
                    </div>
                  </div>
                )}
                {Boolean(image.model_params?.xyz_z_type) && (
                  <div className="flex items-start gap-2">
                    <span className="text-gray-400 font-medium min-w-16">Z Axis:</span>
                    <div>
                      <span className="text-yellow-400">{String(image.model_params?.xyz_z_type)}</span>
                      {Boolean(image.model_params?.xyz_z_values) && (
                        <p className="text-sm text-gray-300 mt-1 font-mono">{String(image.model_params?.xyz_z_values)}</p>
                      )}
                    </div>
                  </div>
                )}
              </div>
            </div>
          )}

          {/* Positive Prompt */}
          {image.positive_prompt && (
            <div>
              <div className="flex items-center justify-between mb-2">
                <h3 className="text-lg font-medium text-white">Positive Prompt</h3>
                <button
                  onClick={() => copyToClipboard(image.positive_prompt!)}
                  className="text-gray-400 hover:text-white p-1"
                  title="Copy to clipboard"
                >
                  <Copy size={18} />
                </button>
              </div>
              <p className="text-sm text-gray-300 bg-gray-800 p-3 rounded-lg whitespace-pre-wrap max-h-48 overflow-y-auto">
                {image.positive_prompt}
              </p>
            </div>
          )}

          {/* Negative Prompt */}
          {image.negative_prompt && (
            <div>
              <div className="flex items-center justify-between mb-2">
                <h3 className="text-lg font-medium text-white">Negative Prompt</h3>
                <button
                  onClick={() => copyToClipboard(image.negative_prompt!)}
                  className="text-gray-400 hover:text-white p-1"
                  title="Copy to clipboard"
                >
                  <Copy size={18} />
                </button>
              </div>
              <p className="text-sm text-gray-300 bg-gray-800 p-3 rounded-lg whitespace-pre-wrap max-h-32 overflow-y-auto">
                {image.negative_prompt}
              </p>
            </div>
          )}

          {/* LoRAs */}
          {image.loras.length > 0 && (
            <div>
              <h3 className="text-lg font-medium text-white mb-2">LoRAs</h3>
              <div className="space-y-2">
                {image.loras.map((lora, index) => (
                  <div key={index} className="flex items-center justify-between bg-gray-800 px-3 py-2 rounded-lg">
                    <span className="text-gray-200">{lora.name}</span>
                    <span className="text-gray-400 text-sm">
                      {lora.weight}
                      {lora.weight_clip !== undefined && lora.weight_clip !== lora.weight && (
                        <span className="ml-1">/ {lora.weight_clip}</span>
                      )}
                    </span>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* ControlNets */}
          {image.controlnets.length > 0 && (
            <div>
              <h3 className="text-lg font-medium text-white mb-2">ControlNets</h3>
              <div className="space-y-2">
                {image.controlnets.map((cn, index) => (
                  <div key={index} className="bg-gray-800 px-3 py-2 rounded-lg">
                    <div className="flex items-center justify-between">
                      <span className="text-gray-200">{cn.model}</span>
                      <span className="text-gray-400 text-sm">{cn.weight}</span>
                    </div>
                    {(cn.guidance_start > 0 || cn.guidance_end < 1) && (
                      <div className="text-xs text-gray-500 mt-1">
                        Range: {cn.guidance_start} - {cn.guidance_end}
                      </div>
                    )}
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* User Tags */}
          <TagEditor
            tags={image.user_tags}
            onChange={(tags) => updateMutation.mutate({ user_tags: tags })}
          />

          {/* User Memo */}
          <MemoEditor
            memo={image.user_memo}
            onChange={(memo) => updateMutation.mutate({ user_memo: memo ?? undefined })}
          />

          {/* Timestamps */}
          <div className="text-sm text-gray-500 pt-4 border-t border-gray-700">
            <p>Created: {new Date(image.created_at).toLocaleString()}</p>
            <p>Updated: {new Date(image.updated_at).toLocaleString()}</p>
          </div>
        </div>
      </div>

      {/* Lightbox Modal */}
      {isLightboxOpen && (
        <div
          className="fixed inset-0 z-50 bg-black/90 flex items-center justify-center"
          onClick={() => setIsLightboxOpen(false)}
        >
          <button
            onClick={() => setIsLightboxOpen(false)}
            className="absolute top-4 right-4 p-2 text-white hover:text-gray-300 transition-colors"
            title="Close"
          >
            <X size={32} />
          </button>
          <img
            src={`/storage/${image.storage_path}`}
            alt={image.model_name || 'Generated image'}
            className="max-w-[95vw] max-h-[95vh] object-contain"
            onClick={(e) => e.stopPropagation()}
          />
        </div>
      )}
    </div>
  )
}

function InfoRow({ label, value }: { label: string; value: string | null | undefined }) {
  if (!value) return null
  return (
    <div className="flex items-center justify-between border-b border-gray-700 pb-2">
      <span className="text-gray-400">{label}</span>
      <span className="text-white font-mono text-sm">{value}</span>
    </div>
  )
}
