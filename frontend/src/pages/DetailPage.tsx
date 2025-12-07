import { useParams, useNavigate } from 'react-router-dom'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { ArrowLeft, Star, Heart, Trash2, Copy } from 'lucide-react'
import toast from 'react-hot-toast'
import { imagesApi } from '@/api/images'
import type { ImageUpdate } from '@/types/image'

export default function DetailPage() {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const queryClient = useQueryClient()

  const { data: image, isLoading, error } = useQuery({
    queryKey: ['image', id],
    queryFn: () => imagesApi.get(id!),
    enabled: !!id,
  })

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
      <button
        onClick={() => navigate(-1)}
        className="flex items-center gap-2 text-gray-400 hover:text-white mb-6"
      >
        <ArrowLeft size={20} />
        <span>Back</span>
      </button>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
        <div>
          <img
            src={`/storage/${image.storage_path}`}
            alt={image.model_name || 'Generated image'}
            className="w-full rounded-lg"
          />
        </div>

        <div className="space-y-6">
          <div className="flex items-center gap-4">
            <div className="flex items-center gap-1">
              {[1, 2, 3, 4, 5].map((star) => (
                <button
                  key={star}
                  onClick={() => updateMutation.mutate({ rating: star })}
                  className="p-1"
                >
                  <Star
                    size={24}
                    className={star <= image.rating ? 'text-yellow-400' : 'text-gray-600'}
                    fill={star <= image.rating ? 'currentColor' : 'none'}
                  />
                </button>
              ))}
            </div>
            <button
              onClick={() => updateMutation.mutate({ is_favorite: !image.is_favorite })}
              className="p-2"
            >
              <Heart
                size={24}
                className={image.is_favorite ? 'text-red-500' : 'text-gray-600'}
                fill={image.is_favorite ? 'currentColor' : 'none'}
              />
            </button>
            <button
              onClick={() => deleteMutation.mutate()}
              className="p-2 text-gray-600 hover:text-red-500"
            >
              <Trash2 size={24} />
            </button>
          </div>

          <div className="space-y-4">
            <InfoRow label="Model" value={image.model_name} />
            <InfoRow label="Model Type" value={image.model_type} />
            <InfoRow label="Source" value={image.source_tool} />
            <InfoRow label="Size" value={`${image.width} x ${image.height}`} />
            <InfoRow label="Sampler" value={image.sampler_name} />
            <InfoRow label="Scheduler" value={image.scheduler} />
            <InfoRow label="Steps" value={image.steps?.toString()} />
            <InfoRow label="CFG Scale" value={image.cfg_scale?.toString()} />
            <InfoRow label="Seed" value={image.seed?.toString()} />
          </div>

          {image.positive_prompt && (
            <div>
              <div className="flex items-center justify-between mb-2">
                <h3 className="text-lg font-medium">Positive Prompt</h3>
                <button
                  onClick={() => copyToClipboard(image.positive_prompt!)}
                  className="text-gray-400 hover:text-white"
                >
                  <Copy size={18} />
                </button>
              </div>
              <p className="text-sm text-gray-300 bg-gray-800 p-3 rounded-lg whitespace-pre-wrap">
                {image.positive_prompt}
              </p>
            </div>
          )}

          {image.negative_prompt && (
            <div>
              <div className="flex items-center justify-between mb-2">
                <h3 className="text-lg font-medium">Negative Prompt</h3>
                <button
                  onClick={() => copyToClipboard(image.negative_prompt!)}
                  className="text-gray-400 hover:text-white"
                >
                  <Copy size={18} />
                </button>
              </div>
              <p className="text-sm text-gray-300 bg-gray-800 p-3 rounded-lg whitespace-pre-wrap">
                {image.negative_prompt}
              </p>
            </div>
          )}

          {image.loras.length > 0 && (
            <div>
              <h3 className="text-lg font-medium mb-2">LoRAs</h3>
              <div className="space-y-2">
                {image.loras.map((lora, index) => (
                  <div key={index} className="flex items-center justify-between bg-gray-800 p-2 rounded">
                    <span>{lora.name}</span>
                    <span className="text-gray-400">{lora.weight}</span>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  )
}

function InfoRow({ label, value }: { label: string; value: string | null | undefined }) {
  if (!value) return null
  return (
    <div className="flex items-center justify-between border-b border-gray-700 pb-2">
      <span className="text-gray-400">{label}</span>
      <span className="text-white">{value}</span>
    </div>
  )
}
