import { useState } from 'react'
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
  Hash,
  Download,
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
import { lorasApi } from '@/api/models'

const RATING_COLORS: Record<number, string> = {
  0: '#6b7280',
  1: '#ef4444',
  2: '#f97316',
  3: '#eab308',
  4: '#22c55e',
  5: '#3b82f6',
}

function formatFileSize(sizeKb: number | null): string {
  if (!sizeKb) return '-'
  if (sizeKb < 1024) return `${sizeKb.toFixed(0)} KB`
  const sizeMb = sizeKb / 1024
  if (sizeMb < 1024) return `${sizeMb.toFixed(1)} MB`
  return `${(sizeMb / 1024).toFixed(2)} GB`
}

// Helper to convert HTML to plain text with preserved line breaks
function htmlToText(html: string | null): string {
  if (!html) return ''
  return html
    // Convert block elements to line breaks
    .replace(/<\/p>/gi, '\n\n')
    .replace(/<br\s*\/?>/gi, '\n')
    .replace(/<\/div>/gi, '\n')
    .replace(/<\/li>/gi, '\n')
    .replace(/<\/h[1-6]>/gi, '\n\n')
    // Remove remaining HTML tags
    .replace(/<[^>]*>/g, '')
    // Decode common HTML entities
    .replace(/&nbsp;/g, ' ')
    .replace(/&amp;/g, '&')
    .replace(/&lt;/g, '<')
    .replace(/&gt;/g, '>')
    .replace(/&quot;/g, '"')
    .replace(/&#39;/g, "'")
    // Clean up excessive whitespace while preserving intentional line breaks
    .replace(/[ \t]+/g, ' ')
    .replace(/\n /g, '\n')
    .replace(/ \n/g, '\n')
    .replace(/\n{3,}/g, '\n\n')
    .trim()
}

export default function LoraDetailPage() {
  const { name } = useParams<{ name: string }>()
  const navigate = useNavigate()
  const decodedName = name ? decodeURIComponent(name) : ''

  const {
    data: detail,
    isLoading: detailLoading,
    error: detailError,
  } = useQuery({
    queryKey: ['lora-detail', decodedName],
    queryFn: () => lorasApi.getDetail(decodedName),
    enabled: !!decodedName,
  })

  const { data: civitai, isLoading: civitaiLoading } = useQuery({
    queryKey: ['lora-civitai', decodedName],
    queryFn: () => lorasApi.getCivitaiInfo(decodedName),
    enabled: !!decodedName,
    staleTime: 1000 * 60 * 60, // 1 hour
  })

  const [selectedVersionIndex, setSelectedVersionIndex] = useState(0)
  const selectedVersion = civitai?.info?.versions?.[selectedVersionIndex]

  if (detailLoading) {
    return (
      <div className="flex items-center justify-center py-16">
        <div className="text-gray-400">Loading LoRA details...</div>
      </div>
    )
  }

  if (detailError || !detail) {
    return (
      <div className="flex items-center justify-center py-16">
        <div className="text-red-400">Failed to load LoRA details</div>
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
            <Layers size={28} className="text-orange-400 shrink-0" />
            <h1 className="text-2xl font-bold truncate" title={detail.display_name}>
              {detail.display_name}
            </h1>
            {detail.hash && (
              <span className="px-2 py-1 bg-green-600/20 text-green-400 text-sm rounded shrink-0 flex items-center gap-1">
                <Hash size={14} />
                Hash available
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
      <div className="grid grid-cols-2 md:grid-cols-5 gap-4">
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
        <div className="bg-gray-800 rounded-lg p-4">
          <div className="flex items-center gap-2 text-gray-400 text-sm mb-1">Avg Weight</div>
          <div className="text-2xl font-bold">
            {detail.avg_weight !== null ? detail.avg_weight.toFixed(2) : '-'}
          </div>
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

        {civitaiLoading && <div className="text-gray-400 py-4">Loading CivitAI information...</div>}

        {civitai && !civitai.found && (
          <div className="text-gray-400 py-4">
            LoRA not found on CivitAI
            {civitai.error && <span className="text-sm ml-2">({civitai.error})</span>}
          </div>
        )}

        {civitai?.info && (
          <div className="space-y-4">
            {/* Compact Header with Link */}
            <div className="flex items-center justify-between flex-wrap gap-2">
              <div className="flex items-center gap-4 text-sm">
                <span className="font-medium text-white">{civitai.info.name}</span>
                {civitai.info.creator && (
                  <span className="text-gray-400">
                    by <span className="text-cyan-400">{civitai.info.creator}</span>
                  </span>
                )}
                {civitai.info.type && (
                  <span className="px-2 py-0.5 bg-gray-700 text-gray-300 text-xs rounded">
                    {civitai.info.type}
                  </span>
                )}
              </div>
              {civitai.info.civitai_url && (
                <a
                  href={civitai.info.civitai_url}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="inline-flex items-center gap-1 px-3 py-1 bg-cyan-600 hover:bg-cyan-700 rounded-lg transition-colors text-sm"
                >
                  <ExternalLink size={14} />
                  View on CivitAI
                </a>
              )}
            </div>

            {/* Version Tabs - Prominent Position */}
            {civitai.info.versions.length > 0 && (
              <div>
                <div className="flex gap-1 overflow-x-auto pb-2">
                  {civitai.info.versions.map((version, idx) => (
                    <button
                      key={version.version_id}
                      onClick={() => setSelectedVersionIndex(idx)}
                      className={`px-3 py-1.5 rounded-lg text-sm whitespace-nowrap transition-colors ${
                        selectedVersionIndex === idx
                          ? 'bg-orange-600 text-white'
                          : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
                      }`}
                    >
                      {version.name}
                      {version.base_model && (
                        <span className="ml-1 text-xs opacity-70">({version.base_model})</span>
                      )}
                    </button>
                  ))}
                </div>

                {/* Selected Version Content */}
                {selectedVersion && (
                  <div className="space-y-4 border-t border-gray-700 pt-4 mt-2">
                    {/* Version Images - Large and Prominent */}
                    {selectedVersion.images.length > 0 && (
                      <div className="flex gap-3 overflow-x-auto pb-2">
                        {selectedVersion.images.map((img, idx) => (
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
                              className="h-48 w-auto rounded-lg object-cover hover:opacity-80 transition-opacity"
                            />
                          </a>
                        ))}
                      </div>
                    )}

                    {/* Description - Version-specific or fallback to model description */}
                    {(selectedVersion.description || civitai.info.description) && (
                      <div className="bg-gray-700/30 rounded-lg p-3">
                        <div className="text-sm font-medium text-gray-400 mb-2">
                          Description
                          {selectedVersion.description && (
                            <span className="text-xs text-orange-400 ml-2">(Version specific)</span>
                          )}
                        </div>
                        <div className="text-gray-300 text-sm leading-relaxed whitespace-pre-wrap max-h-40 overflow-y-auto">
                          {htmlToText(selectedVersion.description || civitai.info.description)}
                        </div>
                      </div>
                    )}

                    {/* Version Info */}
                    <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
                      {selectedVersion.base_model && (
                        <div>
                          <div className="text-gray-400">Base Model</div>
                          <div className="font-medium">{selectedVersion.base_model}</div>
                        </div>
                      )}
                      {selectedVersion.file_size_kb && (
                        <div>
                          <div className="text-gray-400">File Size</div>
                          <div className="font-medium">
                            {formatFileSize(selectedVersion.file_size_kb)}
                          </div>
                        </div>
                      )}
                      {selectedVersion.published_at && (
                        <div>
                          <div className="text-gray-400">Published</div>
                          <div className="font-medium">
                            {new Date(selectedVersion.published_at).toLocaleDateString()}
                          </div>
                        </div>
                      )}
                    </div>

                    {/* Trigger Words */}
                    {selectedVersion.trigger_words.length > 0 && (
                      <div className="bg-gray-700/50 rounded-lg p-3">
                        <div className="text-sm font-medium text-orange-400 mb-2">Trigger Words</div>
                        <div className="flex flex-wrap gap-2">
                          {selectedVersion.trigger_words.map((word, idx) => (
                            <span
                              key={idx}
                              className="px-2 py-1 bg-orange-600/20 text-orange-300 text-sm rounded"
                            >
                              {word}
                            </span>
                          ))}
                        </div>
                      </div>
                    )}

                    {/* Recommended Settings */}
                    {selectedVersion.recommended_settings && (
                      <div className="bg-gray-700/50 rounded-lg p-3">
                        <div className="text-sm font-medium text-cyan-400 mb-2">
                          Recommended Settings
                        </div>
                        <div className="grid grid-cols-2 md:grid-cols-4 gap-3 text-sm">
                          {selectedVersion.recommended_settings.strength && (
                            <div>
                              <span className="text-gray-400">Weight:</span>{' '}
                              <span className="font-medium">
                                {selectedVersion.recommended_settings.strength}
                              </span>
                            </div>
                          )}
                          {selectedVersion.recommended_settings.clip_skip && (
                            <div>
                              <span className="text-gray-400">Clip Skip:</span>{' '}
                              <span className="font-medium">
                                {selectedVersion.recommended_settings.clip_skip}
                              </span>
                            </div>
                          )}
                          {selectedVersion.recommended_settings.steps && (
                            <div>
                              <span className="text-gray-400">Steps:</span>{' '}
                              <span className="font-medium">
                                {selectedVersion.recommended_settings.steps}
                              </span>
                            </div>
                          )}
                          {selectedVersion.recommended_settings.cfg_scale && (
                            <div>
                              <span className="text-gray-400">CFG:</span>{' '}
                              <span className="font-medium">
                                {selectedVersion.recommended_settings.cfg_scale}
                              </span>
                            </div>
                          )}
                          {selectedVersion.recommended_settings.sampler && (
                            <div>
                              <span className="text-gray-400">Sampler:</span>{' '}
                              <span className="font-medium">
                                {selectedVersion.recommended_settings.sampler}
                              </span>
                            </div>
                          )}
                        </div>
                      </div>
                    )}

                    {/* Download Link */}
                    {selectedVersion.download_url && (
                      <a
                        href={selectedVersion.download_url}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="inline-flex items-center gap-2 px-4 py-2 bg-green-600 hover:bg-green-700 rounded-lg transition-colors text-sm"
                      >
                        <Download size={16} />
                        Download {selectedVersion.name}
                        {selectedVersion.file_size_kb && (
                          <span className="text-green-200">
                            ({formatFileSize(selectedVersion.file_size_kb)})
                          </span>
                        )}
                      </a>
                    )}
                  </div>
                )}
              </div>
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

      {/* Top Models & Samplers */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {/* Top Models */}
        {detail.top_models.length > 0 && (
          <div className="bg-gray-800 rounded-lg p-4">
            <h2 className="text-lg font-semibold mb-4 flex items-center gap-2">
              <Box size={18} className="text-blue-400" />
              Top Models Used With
            </h2>
            <div className="space-y-2">
              {detail.top_models.map((model, idx) => (
                <Link
                  key={model.name}
                  to={`/models/${encodeURIComponent(model.name)}`}
                  className="flex items-center justify-between py-2 border-b border-gray-700 last:border-0 hover:bg-gray-700/50 rounded px-2 -mx-2 transition-colors"
                >
                  <div className="flex items-center gap-2 min-w-0">
                    <span className="text-gray-500 w-6 shrink-0">{idx + 1}.</span>
                    <span className="truncate">{model.name}</span>
                  </div>
                  <div className="flex items-center gap-3 text-sm text-gray-400 shrink-0">
                    <span>{model.count} images</span>
                    {model.avg_rating !== null && (
                      <span className="text-yellow-400">★{model.avg_rating.toFixed(2)}</span>
                    )}
                  </div>
                </Link>
              ))}
            </div>
          </div>
        )}

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
      </div>

      {/* View Images Button */}
      <div className="flex justify-center">
        <Link
          to={`/?lora_name=${encodeURIComponent(detail.name)}`}
          className="px-6 py-3 bg-orange-600 hover:bg-orange-700 rounded-lg transition-colors flex items-center gap-2"
        >
          <Image size={20} />
          View {detail.image_count} images with this LoRA
        </Link>
      </div>
    </div>
  )
}
