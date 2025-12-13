import { useParams, useNavigate, Link } from 'react-router-dom'
import { useQuery } from '@tanstack/react-query'
import {
  ArrowLeft,
  Star,
  Image,
  ExternalLink,
  AlertTriangle,
  Box,
  Layers,
} from 'lucide-react'
import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  Cell,
} from 'recharts'
import { modelsApi } from '@/api/models'

const RATING_COLORS: Record<number, string> = {
  0: '#6b7280',
  1: '#ef4444',
  2: '#f97316',
  3: '#eab308',
  4: '#22c55e',
  5: '#3b82f6',
}

export default function ModelDetailPage() {
  const { name } = useParams<{ name: string }>()
  const navigate = useNavigate()
  const decodedName = name ? decodeURIComponent(name) : ''

  const { data: detail, isLoading: detailLoading, error: detailError } = useQuery({
    queryKey: ['model-detail', decodedName],
    queryFn: () => modelsApi.getDetail(decodedName),
    enabled: !!decodedName,
  })

  const { data: civitai, isLoading: civitaiLoading } = useQuery({
    queryKey: ['model-civitai', decodedName],
    queryFn: () => modelsApi.getCivitaiInfo(decodedName),
    enabled: !!decodedName,
    staleTime: 1000 * 60 * 60, // 1 hour
  })

  if (detailLoading) {
    return (
      <div className="flex items-center justify-center py-16">
        <div className="text-gray-400">Loading model details...</div>
      </div>
    )
  }

  if (detailError || !detail) {
    return (
      <div className="flex items-center justify-center py-16">
        <div className="text-red-400">Failed to load model details</div>
      </div>
    )
  }

  const ratingData = Object.entries(detail.rating_distribution).map(([rating, count]) => ({
    rating: parseInt(rating),
    count,
  }))

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-start gap-4">
        <button
          onClick={() => navigate(-1)}
          className="p-2 hover:bg-gray-700 rounded-lg transition-colors"
        >
          <ArrowLeft size={24} />
        </button>
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-3 mb-2">
            <Box size={28} className="text-blue-400 shrink-0" />
            <h1 className="text-2xl font-bold truncate" title={detail.display_name}>
              {detail.display_name}
            </h1>
            {detail.model_type && (
              <span className="px-2 py-1 bg-blue-600/20 text-blue-400 text-sm rounded shrink-0">
                {detail.model_type.toUpperCase()}
              </span>
            )}
          </div>
          {detail.name !== detail.display_name && (
            <p className="text-gray-400 text-sm truncate" title={detail.name}>
              {detail.name}
            </p>
          )}
        </div>
      </div>

      {/* Stats Overview */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        <div className="bg-gray-800 rounded-lg p-4">
          <div className="flex items-center gap-2 text-gray-400 text-sm mb-1">
            <Image size={16} />
            Total Images
          </div>
          <div className="text-2xl font-bold">{detail.image_count.toLocaleString()}</div>
        </div>
        <div className="bg-gray-800 rounded-lg p-4">
          <div className="flex items-center gap-2 text-gray-400 text-sm mb-1">
            <Star size={16} />
            Rated
          </div>
          <div className="text-2xl font-bold">{detail.rated_count.toLocaleString()}</div>
        </div>
        <div className="bg-gray-800 rounded-lg p-4">
          <div className="flex items-center gap-2 text-gray-400 text-sm mb-1">
            <Star size={16} className="text-yellow-400" />
            Avg Rating
          </div>
          <div className="text-2xl font-bold">
            {detail.avg_rating !== null ? `★${detail.avg_rating.toFixed(2)}` : '-'}
          </div>
        </div>
        <div className="bg-gray-800 rounded-lg p-4">
          <div className="flex items-center gap-2 text-gray-400 text-sm mb-1">
            <Star size={16} className="text-green-400" />
            High Rated (4+)
          </div>
          <div className="text-2xl font-bold">{detail.high_rated_count.toLocaleString()}</div>
        </div>
      </div>

      {/* CivitAI Info */}
      <div className="bg-gray-800 rounded-lg p-4">
        <h2 className="text-lg font-semibold mb-4 flex items-center gap-2">
          <ExternalLink size={20} className="text-cyan-400" />
          CivitAI Information
          {civitai?.info && !civitai.info.is_exact_match && (
            <span className="px-2 py-0.5 bg-yellow-600/20 text-yellow-400 text-xs rounded flex items-center gap-1">
              <AlertTriangle size={12} />
              Fuzzy match
            </span>
          )}
        </h2>

        {civitaiLoading && (
          <div className="text-gray-400 py-4">Loading CivitAI information...</div>
        )}

        {civitai && !civitai.found && (
          <div className="text-gray-400 py-4">
            Model not found on CivitAI
            {civitai.error && <span className="text-sm ml-2">({civitai.error})</span>}
          </div>
        )}

        {civitai?.info && (
          <div className="space-y-4">
            {/* Images */}
            {civitai.info.images.length > 0 && (
              <div className="flex gap-2 overflow-x-auto pb-2">
                {civitai.info.images.map((img, idx) => (
                  <a
                    key={idx}
                    href={img.url}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="shrink-0"
                  >
                    <img
                      src={img.url}
                      alt={`Preview ${idx + 1}`}
                      className="h-32 w-auto rounded-lg object-cover hover:opacity-80 transition-opacity"
                    />
                  </a>
                ))}
              </div>
            )}

            {/* Info Grid */}
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div>
                <div className="text-gray-400 text-sm">Name on CivitAI</div>
                <div className="font-medium">{civitai.info.name}</div>
              </div>
              {civitai.info.creator && (
                <div>
                  <div className="text-gray-400 text-sm">Creator</div>
                  <div className="font-medium">{civitai.info.creator}</div>
                </div>
              )}
              {civitai.info.base_model && (
                <div>
                  <div className="text-gray-400 text-sm">Base Model</div>
                  <div className="font-medium">{civitai.info.base_model}</div>
                </div>
              )}
              {civitai.info.type && (
                <div>
                  <div className="text-gray-400 text-sm">Type</div>
                  <div className="font-medium">{civitai.info.type}</div>
                </div>
              )}
            </div>

            {/* Recommended Settings */}
            {civitai.info.recommended_settings && (
              <div className="bg-gray-700/50 rounded-lg p-3">
                <div className="text-sm font-medium text-cyan-400 mb-2">Recommended Settings</div>
                <div className="grid grid-cols-2 md:grid-cols-4 gap-3 text-sm">
                  {civitai.info.recommended_settings.clip_skip && (
                    <div>
                      <span className="text-gray-400">Clip Skip:</span>{' '}
                      <span className="font-medium">{civitai.info.recommended_settings.clip_skip}</span>
                    </div>
                  )}
                  {civitai.info.recommended_settings.steps && (
                    <div>
                      <span className="text-gray-400">Steps:</span>{' '}
                      <span className="font-medium">{civitai.info.recommended_settings.steps}</span>
                    </div>
                  )}
                  {civitai.info.recommended_settings.cfg_scale && (
                    <div>
                      <span className="text-gray-400">CFG:</span>{' '}
                      <span className="font-medium">{civitai.info.recommended_settings.cfg_scale}</span>
                    </div>
                  )}
                  {civitai.info.recommended_settings.sampler && (
                    <div>
                      <span className="text-gray-400">Sampler:</span>{' '}
                      <span className="font-medium">{civitai.info.recommended_settings.sampler}</span>
                    </div>
                  )}
                  {civitai.info.recommended_settings.vae && (
                    <div>
                      <span className="text-gray-400">VAE:</span>{' '}
                      <span className="font-medium">{civitai.info.recommended_settings.vae}</span>
                    </div>
                  )}
                </div>
              </div>
            )}

            {/* CivitAI Link */}
            {civitai.info.civitai_url && (
              <a
                href={civitai.info.civitai_url}
                target="_blank"
                rel="noopener noreferrer"
                className="inline-flex items-center gap-2 px-4 py-2 bg-cyan-600 hover:bg-cyan-700 rounded-lg transition-colors"
              >
                <ExternalLink size={16} />
                View on CivitAI
              </a>
            )}
          </div>
        )}
      </div>

      {/* Rating Distribution */}
      <div className="bg-gray-800 rounded-lg p-4">
        <h2 className="text-lg font-semibold mb-4">Rating Distribution</h2>
        <ResponsiveContainer width="100%" height={200}>
          <BarChart data={ratingData}>
            <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
            <XAxis
              dataKey="rating"
              stroke="#9ca3af"
              fontSize={12}
              tickFormatter={(value) => `★${value}`}
            />
            <YAxis stroke="#9ca3af" fontSize={12} />
            <Tooltip
              contentStyle={{ backgroundColor: '#1f2937', border: '1px solid #374151' }}
              labelFormatter={(value) => `Rating: ★${value}`}
            />
            <Bar dataKey="count" radius={[4, 4, 0, 0]}>
              {ratingData.map((entry) => (
                <Cell key={entry.rating} fill={RATING_COLORS[entry.rating] || '#6b7280'} />
              ))}
            </Bar>
          </BarChart>
        </ResponsiveContainer>
      </div>

      {/* Top Samplers & LoRAs */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {/* Top Samplers */}
        {detail.top_samplers.length > 0 && (
          <div className="bg-gray-800 rounded-lg p-4">
            <h2 className="text-lg font-semibold mb-4">Top Samplers</h2>
            <div className="space-y-2">
              {detail.top_samplers.map((sampler, idx) => (
                <div
                  key={sampler.name}
                  className="flex items-center justify-between py-2 border-b border-gray-700 last:border-0"
                >
                  <div className="flex items-center gap-2">
                    <span className="text-gray-500 w-6">{idx + 1}.</span>
                    <span className="truncate">{sampler.name}</span>
                  </div>
                  <div className="flex items-center gap-3 text-sm text-gray-400 shrink-0">
                    <span>{sampler.count} images</span>
                    {sampler.avg_rating !== null && (
                      <span className="text-yellow-400">★{sampler.avg_rating.toFixed(2)}</span>
                    )}
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Top LoRAs */}
        {detail.top_loras.length > 0 && (
          <div className="bg-gray-800 rounded-lg p-4">
            <h2 className="text-lg font-semibold mb-4 flex items-center gap-2">
              <Layers size={18} className="text-orange-400" />
              Top LoRAs Used
            </h2>
            <div className="space-y-2">
              {detail.top_loras.map((lora, idx) => (
                <Link
                  key={lora.name}
                  to={`/loras/${encodeURIComponent(lora.name)}`}
                  className="flex items-center justify-between py-2 border-b border-gray-700 last:border-0 hover:bg-gray-700/50 rounded px-2 -mx-2 transition-colors"
                >
                  <div className="flex items-center gap-2 min-w-0">
                    <span className="text-gray-500 w-6 shrink-0">{idx + 1}.</span>
                    <span className="truncate">{lora.name}</span>
                  </div>
                  <div className="flex items-center gap-3 text-sm text-gray-400 shrink-0">
                    <span>{lora.count} images</span>
                    {lora.avg_rating !== null && (
                      <span className="text-yellow-400">★{lora.avg_rating.toFixed(2)}</span>
                    )}
                  </div>
                </Link>
              ))}
            </div>
          </div>
        )}
      </div>

      {/* View Images Button */}
      <div className="flex justify-center">
        <Link
          to={`/?model_name=${encodeURIComponent(detail.name)}`}
          className="px-6 py-3 bg-blue-600 hover:bg-blue-700 rounded-lg transition-colors flex items-center gap-2"
        >
          <Image size={20} />
          View {detail.image_count} images with this model
        </Link>
      </div>
    </div>
  )
}
