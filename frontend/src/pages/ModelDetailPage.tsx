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
  Download,
  FileText,
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

// Helper to format file size
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

export default function ModelDetailPage() {
  const { name } = useParams<{ name: string }>()
  const navigate = useNavigate()
  const decodedName = name ? decodeURIComponent(name) : ''
  const [selectedCivitaiVersionIndex, setSelectedCivitaiVersionIndex] = useState(0)
  const [selectedLocalVersionIndex, setSelectedLocalVersionIndex] = useState<number | null>(null) // null = show all

  const {
    data: detail,
    isLoading: detailLoading,
    error: detailError,
  } = useQuery({
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

        {civitaiLoading && <div className="text-gray-400 py-4">Loading CivitAI information...</div>}

        {civitai && !civitai.found && (
          <div className="text-gray-400 py-4">
            Model not found on CivitAI
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
                      onClick={() => setSelectedCivitaiVersionIndex(idx)}
                      className={`px-3 py-1.5 rounded-lg text-sm whitespace-nowrap transition-colors ${
                        selectedCivitaiVersionIndex === idx
                          ? 'bg-cyan-600 text-white'
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
                {(() => {
                  const version = civitai.info.versions[selectedCivitaiVersionIndex]
                  if (!version) return null

                  return (
                    <div className="space-y-4 border-t border-gray-700 pt-4 mt-2">
                      {/* Version Images - Large and Prominent */}
                      {version.images.length > 0 && (
                        <div className="flex gap-3 overflow-x-auto pb-2">
                          {version.images.map((img, imgIdx) => (
                            <a
                              key={imgIdx}
                              href={img.url}
                              target="_blank"
                              rel="noopener noreferrer"
                              className="shrink-0"
                            >
                              <img
                                src={img.url}
                                alt={`Preview ${imgIdx + 1}`}
                                className="h-48 w-auto rounded-lg object-cover hover:opacity-80 transition-opacity"
                              />
                            </a>
                          ))}
                        </div>
                      )}

                      {/* Description - Version-specific or fallback to model description */}
                      {(version.description || civitai.info.description) && (
                        <div className="bg-gray-700/30 rounded-lg p-3">
                          <div className="flex items-center gap-2 text-gray-400 text-sm mb-2">
                            <FileText size={14} />
                            Description
                            {version.description && (
                              <span className="text-xs text-cyan-400">(Version specific)</span>
                            )}
                          </div>
                          <div className="text-sm text-gray-300 whitespace-pre-wrap max-h-40 overflow-y-auto">
                            {htmlToText(version.description || civitai.info.description)}
                          </div>
                        </div>
                      )}

                      {/* Version Info */}
                      <div className="grid grid-cols-2 md:grid-cols-4 gap-3 text-sm">
                        {version.base_model && (
                          <div>
                            <span className="text-gray-400">Base Model:</span>{' '}
                            <span className="font-medium">{version.base_model}</span>
                          </div>
                        )}
                        {version.file_size_kb && (
                          <div>
                            <span className="text-gray-400">File Size:</span>{' '}
                            <span className="font-medium">{formatFileSize(version.file_size_kb)}</span>
                          </div>
                        )}
                        {version.published_at && (
                          <div>
                            <span className="text-gray-400">Published:</span>{' '}
                            <span className="font-medium">
                              {new Date(version.published_at).toLocaleDateString()}
                            </span>
                          </div>
                        )}
                      </div>

                      {/* Trigger Words (for LoRA) */}
                      {version.trigger_words.length > 0 && (
                        <div>
                          <div className="text-gray-400 text-sm mb-1">Trigger Words</div>
                          <div className="flex flex-wrap gap-1">
                            {version.trigger_words.map((word, idx) => (
                              <span
                                key={idx}
                                className="px-2 py-0.5 bg-purple-600/20 text-purple-300 text-sm rounded"
                              >
                                {word}
                              </span>
                            ))}
                          </div>
                        </div>
                      )}

                      {/* Recommended Settings */}
                      {version.recommended_settings && (
                        <div className="bg-gray-700/50 rounded-lg p-3">
                          <div className="text-sm font-medium text-cyan-400 mb-2">
                            Recommended Settings
                          </div>
                          <div className="grid grid-cols-2 md:grid-cols-4 gap-3 text-sm">
                            {version.recommended_settings.clip_skip && (
                              <div>
                                <span className="text-gray-400">Clip Skip:</span>{' '}
                                <span className="font-medium">
                                  {version.recommended_settings.clip_skip}
                                </span>
                              </div>
                            )}
                            {version.recommended_settings.steps && (
                              <div>
                                <span className="text-gray-400">Steps:</span>{' '}
                                <span className="font-medium">
                                  {version.recommended_settings.steps}
                                </span>
                              </div>
                            )}
                            {version.recommended_settings.cfg_scale && (
                              <div>
                                <span className="text-gray-400">CFG:</span>{' '}
                                <span className="font-medium">
                                  {version.recommended_settings.cfg_scale}
                                </span>
                              </div>
                            )}
                            {version.recommended_settings.sampler && (
                              <div>
                                <span className="text-gray-400">Sampler:</span>{' '}
                                <span className="font-medium">
                                  {version.recommended_settings.sampler}
                                </span>
                              </div>
                            )}
                            {version.recommended_settings.vae && (
                              <div>
                                <span className="text-gray-400">VAE:</span>{' '}
                                <span className="font-medium">
                                  {version.recommended_settings.vae}
                                </span>
                              </div>
                            )}
                            {version.recommended_settings.strength && (
                              <div>
                                <span className="text-gray-400">Strength:</span>{' '}
                                <span className="font-medium">
                                  {version.recommended_settings.strength}
                                </span>
                              </div>
                            )}
                          </div>
                        </div>
                      )}

                      {/* Download Link */}
                      {version.download_url && (
                        <a
                          href={version.download_url}
                          target="_blank"
                          rel="noopener noreferrer"
                          className="inline-flex items-center gap-2 px-3 py-1.5 bg-green-600 hover:bg-green-700 rounded-lg transition-colors text-sm"
                        >
                          <Download size={14} />
                          Download this version
                        </a>
                      )}
                    </div>
                  )
                })()}
              </div>
            )}
          </div>
        )}
      </div>

      {/* Local Versions */}
      {detail.versions && detail.versions.length > 1 && (
        <div className="bg-gray-800 rounded-lg p-4">
          <h2 className="text-lg font-semibold mb-4 flex items-center gap-2">
            <Layers size={20} className="text-purple-400" />
            Local Versions ({detail.versions.length})
          </h2>

          {/* Version Tabs */}
          <div className="flex gap-1 overflow-x-auto pb-2 mb-4">
            <button
              onClick={() => setSelectedLocalVersionIndex(null)}
              className={`px-3 py-1.5 rounded-lg text-sm whitespace-nowrap transition-colors ${
                selectedLocalVersionIndex === null
                  ? 'bg-purple-600 text-white'
                  : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
              }`}
            >
              All Versions
            </button>
            {detail.versions.map((version, idx) => (
              <button
                key={version.name}
                onClick={() => setSelectedLocalVersionIndex(idx)}
                className={`px-3 py-1.5 rounded-lg text-sm whitespace-nowrap transition-colors ${
                  selectedLocalVersionIndex === idx
                    ? 'bg-purple-600 text-white'
                    : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
                }`}
              >
                {version.display_name}
              </button>
            ))}
          </div>

          {/* Version Stats */}
          {selectedLocalVersionIndex === null ? (
            // Show all versions summary
            <div className="space-y-2">
              {detail.versions.map((version) => (
                <div
                  key={version.name}
                  className="flex items-center justify-between py-2 border-b border-gray-700 last:border-0"
                >
                  <span className="truncate font-medium">{version.display_name}</span>
                  <div className="flex items-center gap-4 text-sm text-gray-400 shrink-0">
                    <span>{version.image_count} images</span>
                    {version.avg_rating !== null && (
                      <span className="text-yellow-400">★{version.avg_rating.toFixed(2)}</span>
                    )}
                    <span className="text-green-400">{version.high_rated_count} high</span>
                  </div>
                </div>
              ))}
            </div>
          ) : (
            // Show selected version details
            (() => {
              const version = detail.versions[selectedLocalVersionIndex]
              if (!version) return null

              const versionRatingData = Object.entries(version.rating_distribution).map(
                ([rating, count]) => ({
                  rating: parseInt(rating),
                  count,
                })
              )

              return (
                <div className="space-y-4">
                  {/* Version Stats */}
                  <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
                    <div className="bg-gray-700/50 rounded-lg p-3">
                      <div className="text-gray-400 text-sm">Images</div>
                      <div className="text-xl font-bold">{version.image_count.toLocaleString()}</div>
                    </div>
                    <div className="bg-gray-700/50 rounded-lg p-3">
                      <div className="text-gray-400 text-sm">Rated</div>
                      <div className="text-xl font-bold">{version.rated_count.toLocaleString()}</div>
                    </div>
                    <div className="bg-gray-700/50 rounded-lg p-3">
                      <div className="text-gray-400 text-sm">Avg Rating</div>
                      <div className="text-xl font-bold">
                        {version.avg_rating !== null ? `★${version.avg_rating.toFixed(2)}` : '-'}
                      </div>
                    </div>
                    <div className="bg-gray-700/50 rounded-lg p-3">
                      <div className="text-gray-400 text-sm">High Rated</div>
                      <div className="text-xl font-bold">{version.high_rated_count.toLocaleString()}</div>
                    </div>
                  </div>

                  {/* Version Rating Distribution */}
                  <div>
                    <div className="text-gray-400 text-sm mb-2">Rating Distribution</div>
                    <ResponsiveContainer width="100%" height={150}>
                      <BarChart data={versionRatingData}>
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
                          {versionRatingData.map((entry) => (
                            <Cell key={entry.rating} fill={RATING_COLORS[entry.rating] || '#6b7280'} />
                          ))}
                        </Bar>
                      </BarChart>
                    </ResponsiveContainer>
                  </div>

                  {/* View Images Link */}
                  <Link
                    to={`/?model_name=${encodeURIComponent(version.name)}`}
                    className="inline-flex items-center gap-2 px-4 py-2 bg-purple-600 hover:bg-purple-700 rounded-lg transition-colors text-sm"
                  >
                    <Image size={16} />
                    View {version.image_count} images with this version
                  </Link>
                </div>
              )
            })()
          )}
        </div>
      )}

      {/* Rating Distribution */}
      <div className="bg-gray-800 rounded-lg p-4">
        <h2 className="text-lg font-semibold mb-4">Rating Distribution (All Versions)</h2>
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
