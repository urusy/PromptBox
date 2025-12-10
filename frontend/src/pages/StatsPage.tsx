import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  PieChart,
  Pie,
  Cell,
  LineChart,
  Line,
  ComposedChart,
} from 'recharts'
import { Image, Star, Heart, TrendingUp, Sparkles, BarChart2 } from 'lucide-react'
import { statsApi } from '@/api/stats'
import type { RatingAnalysisItem } from '@/types/stats'

const COLORS = ['#3b82f6', '#10b981', '#f59e0b', '#ef4444', '#8b5cf6', '#ec4899', '#14b8a6', '#f97316', '#6366f1', '#84cc16']

const RATING_COLORS: Record<number, string> = {
  0: '#6b7280', // gray
  1: '#ef4444', // red
  2: '#f97316', // orange
  3: '#eab308', // yellow
  4: '#22c55e', // green
  5: '#3b82f6', // blue
}

// Rating Analysis Chart Component
function RatingAnalysisChart({
  data,
  title,
  color
}: {
  data: RatingAnalysisItem[]
  title: string
  color: string
}) {
  if (data.length === 0) return null

  return (
    <div className="bg-gray-800 rounded-lg p-4">
      <h3 className="text-md font-semibold mb-3">{title}</h3>
      <ResponsiveContainer width="100%" height={200}>
        <ComposedChart data={data} layout="vertical">
          <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
          <XAxis type="number" stroke="#9ca3af" fontSize={11} domain={[0, 5]} />
          <YAxis
            type="category"
            dataKey="name"
            stroke="#9ca3af"
            fontSize={10}
            width={100}
            tickFormatter={(value) => value.length > 12 ? value.slice(0, 12) + '...' : value}
          />
          <Tooltip
            contentStyle={{ backgroundColor: '#1f2937', border: '1px solid #374151' }}
            formatter={(value: number, name: string) => {
              if (name === 'avg_rating') return [`★${value.toFixed(2)}`, 'Avg Rating']
              return [value, name]
            }}
            labelFormatter={(label) => label}
          />
          <Bar dataKey="avg_rating" fill={color} radius={[0, 4, 4, 0]} name="avg_rating" />
        </ComposedChart>
      </ResponsiveContainer>
      <div className="mt-2 text-xs text-gray-400 space-y-1">
        {data.slice(0, 3).map((item, i) => (
          <div key={i} className="flex justify-between">
            <span className="truncate mr-2">{i + 1}. {item.name}</span>
            <span className="shrink-0">
              ★{item.avg_rating.toFixed(2)} ({item.count}枚, 高評価{item.high_rated_count}枚)
            </span>
          </div>
        ))}
      </div>
    </div>
  )
}

export default function StatsPage() {
  const [analysisTab, setAnalysisTab] = useState<'overall' | 'by-model'>('overall')
  const [selectedModel, setSelectedModel] = useState<string>('')

  const { data: stats, isLoading, error } = useQuery({
    queryKey: ['stats'],
    queryFn: () => statsApi.get(30),
  })

  const { data: ratingAnalysis } = useQuery({
    queryKey: ['rating-analysis', analysisTab === 'by-model' ? selectedModel : undefined],
    queryFn: () => statsApi.getRatingAnalysis(3, analysisTab === 'by-model' && selectedModel ? selectedModel : undefined),
  })

  const { data: modelList } = useQuery({
    queryKey: ['models-for-analysis'],
    queryFn: () => statsApi.getModelsForAnalysis(3),
  })

  const { data: modelRatingDistribution } = useQuery({
    queryKey: ['model-rating-distribution'],
    queryFn: () => statsApi.getModelRatingDistribution(3, 15),
  })

  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-16">
        <div className="text-gray-400">Loading statistics...</div>
      </div>
    )
  }

  if (error || !stats) {
    return (
      <div className="flex items-center justify-center py-16">
        <div className="text-red-400">Failed to load statistics</div>
      </div>
    )
  }

  return (
    <div className="space-y-8">
      <h1 className="text-2xl font-bold">Statistics</h1>

      {/* Overview Cards */}
      <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-5 gap-6">
        <div className="bg-gray-800 rounded-lg p-4">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-blue-600/20 rounded-lg">
              <Image size={24} className="text-blue-400" />
            </div>
            <div>
              <div className="text-2xl font-bold">{stats.overview.total_images.toLocaleString()}</div>
              <div className="text-sm text-gray-400">Total Images</div>
            </div>
          </div>
        </div>

        <div className="bg-gray-800 rounded-lg p-4">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-red-600/20 rounded-lg">
              <Heart size={24} className="text-red-400" />
            </div>
            <div>
              <div className="text-2xl font-bold">{stats.overview.total_favorites.toLocaleString()}</div>
              <div className="text-sm text-gray-400">Favorites</div>
            </div>
          </div>
        </div>

        <div className="bg-gray-800 rounded-lg p-4">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-yellow-600/20 rounded-lg">
              <Star size={24} className="text-yellow-400" />
            </div>
            <div>
              <div className="text-2xl font-bold">{stats.overview.total_rated.toLocaleString()}</div>
              <div className="text-sm text-gray-400">Rated</div>
            </div>
          </div>
        </div>

        <div className="bg-gray-800 rounded-lg p-4">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-gray-600/20 rounded-lg">
              <Star size={24} className="text-gray-400" />
            </div>
            <div>
              <div className="text-2xl font-bold">{stats.overview.total_unrated.toLocaleString()}</div>
              <div className="text-sm text-gray-400">Unrated</div>
            </div>
          </div>
        </div>

        <div className="bg-gray-800 rounded-lg p-4">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-green-600/20 rounded-lg">
              <TrendingUp size={24} className="text-green-400" />
            </div>
            <div>
              <div className="text-2xl font-bold">
                {stats.overview.avg_rating ? `★${stats.overview.avg_rating}` : '-'}
              </div>
              <div className="text-sm text-gray-400">Avg Rating</div>
            </div>
          </div>
        </div>
      </div>

      {/* Daily Trend */}
      {stats.daily_counts.length > 0 && (
        <div className="bg-gray-800 rounded-lg p-4">
          <h2 className="text-lg font-semibold mb-4">Daily Image Count (Last 30 Days)</h2>
          <ResponsiveContainer width="100%" height={250}>
            <LineChart data={stats.daily_counts}>
              <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
              <XAxis
                dataKey="date"
                stroke="#9ca3af"
                fontSize={12}
                tickFormatter={(value) => value.slice(5)} // MM-DD
              />
              <YAxis stroke="#9ca3af" fontSize={12} />
              <Tooltip
                contentStyle={{ backgroundColor: '#1f2937', border: '1px solid #374151' }}
                labelStyle={{ color: '#fff' }}
              />
              <Line
                type="monotone"
                dataKey="count"
                stroke="#3b82f6"
                strokeWidth={2}
                dot={{ fill: '#3b82f6', strokeWidth: 0, r: 3 }}
              />
            </LineChart>
          </ResponsiveContainer>
        </div>
      )}

      {/* Charts Grid */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Model Type Distribution */}
        {stats.by_model_type.length > 0 && (
          <div className="bg-gray-800 rounded-lg p-4">
            <h2 className="text-lg font-semibold mb-4">By Model Type</h2>
            <ResponsiveContainer width="100%" height={250}>
              <PieChart>
                <Pie
                  data={stats.by_model_type as unknown as Record<string, unknown>[]}
                  dataKey="count"
                  nameKey="name"
                  cx="50%"
                  cy="50%"
                  outerRadius={80}
                  label={({ name, percent }) => `${name} (${((percent ?? 0) * 100).toFixed(0)}%)`}
                  labelLine={{ stroke: '#9ca3af' }}
                >
                  {stats.by_model_type.map((_, index) => (
                    <Cell key={index} fill={COLORS[index % COLORS.length]} />
                  ))}
                </Pie>
                <Tooltip
                  contentStyle={{ backgroundColor: '#1f2937', border: '1px solid #374151' }}
                />
              </PieChart>
            </ResponsiveContainer>
          </div>
        )}

        {/* Source Tool Distribution */}
        {stats.by_source_tool.length > 0 && (
          <div className="bg-gray-800 rounded-lg p-4">
            <h2 className="text-lg font-semibold mb-4">By Source Tool</h2>
            <ResponsiveContainer width="100%" height={250}>
              <PieChart>
                <Pie
                  data={stats.by_source_tool as unknown as Record<string, unknown>[]}
                  dataKey="count"
                  nameKey="name"
                  cx="50%"
                  cy="50%"
                  outerRadius={80}
                  label={({ name, percent }) => `${name} (${((percent ?? 0) * 100).toFixed(0)}%)`}
                  labelLine={{ stroke: '#9ca3af' }}
                >
                  {stats.by_source_tool.map((_, index) => (
                    <Cell key={index} fill={COLORS[index % COLORS.length]} />
                  ))}
                </Pie>
                <Tooltip
                  contentStyle={{ backgroundColor: '#1f2937', border: '1px solid #374151' }}
                />
              </PieChart>
            </ResponsiveContainer>
          </div>
        )}

        {/* Rating Distribution */}
        {stats.by_rating.length > 0 && (
          <div className="bg-gray-800 rounded-lg p-4">
            <h2 className="text-lg font-semibold mb-4">Rating Distribution</h2>
            <ResponsiveContainer width="100%" height={250}>
              <BarChart data={stats.by_rating}>
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
                  {stats.by_rating.map((entry) => (
                    <Cell key={entry.rating} fill={RATING_COLORS[entry.rating] || '#6b7280'} />
                  ))}
                </Bar>
              </BarChart>
            </ResponsiveContainer>
          </div>
        )}

        {/* Top Models */}
        {stats.by_model_name.length > 0 && (
          <div className="bg-gray-800 rounded-lg p-4">
            <h2 className="text-lg font-semibold mb-4">Top Models</h2>
            <ResponsiveContainer width="100%" height={250}>
              <BarChart data={stats.by_model_name} layout="vertical">
                <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
                <XAxis type="number" stroke="#9ca3af" fontSize={12} />
                <YAxis
                  type="category"
                  dataKey="name"
                  stroke="#9ca3af"
                  fontSize={11}
                  width={120}
                  tickFormatter={(value) => value.length > 15 ? value.slice(0, 15) + '...' : value}
                />
                <Tooltip
                  contentStyle={{ backgroundColor: '#1f2937', border: '1px solid #374151' }}
                />
                <Bar dataKey="count" fill="#3b82f6" radius={[0, 4, 4, 0]} />
              </BarChart>
            </ResponsiveContainer>
          </div>
        )}

        {/* Top Samplers */}
        {stats.by_sampler.length > 0 && (
          <div className="bg-gray-800 rounded-lg p-4">
            <h2 className="text-lg font-semibold mb-4">Top Samplers</h2>
            <ResponsiveContainer width="100%" height={250}>
              <BarChart data={stats.by_sampler} layout="vertical">
                <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
                <XAxis type="number" stroke="#9ca3af" fontSize={12} />
                <YAxis
                  type="category"
                  dataKey="name"
                  stroke="#9ca3af"
                  fontSize={11}
                  width={120}
                  tickFormatter={(value) => value.length > 15 ? value.slice(0, 15) + '...' : value}
                />
                <Tooltip
                  contentStyle={{ backgroundColor: '#1f2937', border: '1px solid #374151' }}
                />
                <Bar dataKey="count" fill="#10b981" radius={[0, 4, 4, 0]} />
              </BarChart>
            </ResponsiveContainer>
          </div>
        )}

        {/* Top LoRAs */}
        {stats.by_lora.length > 0 && (
          <div className="bg-gray-800 rounded-lg p-4">
            <h2 className="text-lg font-semibold mb-4">Top LoRAs</h2>
            <ResponsiveContainer width="100%" height={250}>
              <BarChart data={stats.by_lora} layout="vertical">
                <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
                <XAxis type="number" stroke="#9ca3af" fontSize={12} />
                <YAxis
                  type="category"
                  dataKey="name"
                  stroke="#9ca3af"
                  fontSize={11}
                  width={120}
                  tickFormatter={(value) => value.length > 15 ? value.slice(0, 15) + '...' : value}
                />
                <Tooltip
                  contentStyle={{ backgroundColor: '#1f2937', border: '1px solid #374151' }}
                />
                <Bar dataKey="count" fill="#f59e0b" radius={[0, 4, 4, 0]} />
              </BarChart>
            </ResponsiveContainer>
          </div>
        )}
      </div>

      {/* Model Rating Distribution Section */}
      {modelRatingDistribution && modelRatingDistribution.items.length > 0 && (
        <>
          <div className="flex items-center gap-2 mt-8">
            <BarChart2 size={24} className="text-cyan-400" />
            <h2 className="text-xl font-bold">Model Rating Distribution</h2>
            <span className="text-sm text-gray-400 hidden sm:inline">- Average rating by model</span>
          </div>

          <div className="bg-gray-800 rounded-lg p-4">
            <ResponsiveContainer width="100%" height={Math.max(300, modelRatingDistribution.items.length * 35)}>
              <BarChart
                data={[...modelRatingDistribution.items]
                  .sort((a, b) => (b.avg_rating || 0) - (a.avg_rating || 0))
                  .map(item => ({
                    name: item.model_name.length > 20 ? item.model_name.slice(0, 20) + '...' : item.model_name,
                    fullName: item.model_name,
                    avg_rating: item.avg_rating || 0,
                    total: item.total,
                    rated_count: item.total - item.rating_0,
                  }))}
                layout="vertical"
              >
                <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
                <XAxis
                  type="number"
                  stroke="#9ca3af"
                  fontSize={12}
                  domain={[0, 5]}
                  ticks={[0, 1, 2, 3, 4, 5]}
                  tickFormatter={(value) => `★${value}`}
                />
                <YAxis
                  type="category"
                  dataKey="name"
                  stroke="#9ca3af"
                  fontSize={11}
                  width={150}
                />
                <Tooltip
                  contentStyle={{ backgroundColor: '#1f2937', border: '1px solid #374151' }}
                  formatter={(value: number) => [`★${value.toFixed(2)}`, 'Avg Rating']}
                  labelFormatter={(_, payload) => {
                    if (payload && payload[0]) {
                      const data = payload[0].payload
                      return `${data.fullName} (Total: ${data.total}, Rated: ${data.rated_count})`
                    }
                    return ''
                  }}
                />
                <Bar dataKey="avg_rating" fill="#14b8a6" radius={[0, 4, 4, 0]} />
              </BarChart>
            </ResponsiveContainer>
          </div>
        </>
      )}

      {/* Rating Analysis Section */}
      {ratingAnalysis && (
        <>
          <div className="flex items-center gap-2 mt-8">
            <Sparkles size={24} className="text-purple-400" />
            <h2 className="text-xl font-bold">Rating Analysis</h2>
            <span className="text-sm text-gray-400 hidden sm:inline">- Which settings get higher ratings?</span>
          </div>

          {/* Tab Buttons: Overall + Model Names */}
          <div className="flex flex-wrap gap-2">
            <button
              onClick={() => {
                setAnalysisTab('overall')
                setSelectedModel('')
              }}
              className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
                analysisTab === 'overall'
                  ? 'bg-purple-600 text-white'
                  : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
              }`}
            >
              Overall
            </button>
            {modelList?.models.map((model) => (
              <button
                key={model}
                onClick={() => {
                  setAnalysisTab('by-model')
                  setSelectedModel(model)
                }}
                className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
                  analysisTab === 'by-model' && selectedModel === model
                    ? 'bg-purple-600 text-white'
                    : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
                }`}
                title={model}
              >
                {model.length > 15 ? model.slice(0, 15) + '...' : model}
              </button>
            ))}
          </div>

          {/* Selected Model Display */}
          {analysisTab === 'by-model' && selectedModel && (
            <div className="bg-purple-900/30 border border-purple-600/50 rounded-lg px-4 py-2">
              <span className="text-purple-300 text-sm">Analyzing model: </span>
              <span className="text-white font-medium">{selectedModel}</span>
            </div>
          )}

          {/* Analysis Charts */}
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            {analysisTab === 'overall' && (
              <RatingAnalysisChart
                data={ratingAnalysis.by_model}
                title="Best Models"
                color="#3b82f6"
              />
            )}
            <RatingAnalysisChart
              data={ratingAnalysis.by_sampler}
              title={analysisTab === 'by-model' ? `Best Samplers` : 'Best Samplers'}
              color="#10b981"
            />
            <RatingAnalysisChart
              data={ratingAnalysis.by_lora}
              title={analysisTab === 'by-model' ? `Best LoRAs` : 'Best LoRAs'}
              color="#f59e0b"
            />
            <RatingAnalysisChart
              data={ratingAnalysis.by_steps}
              title={analysisTab === 'by-model' ? `Best Steps` : 'Best Steps Range'}
              color="#8b5cf6"
            />
            <RatingAnalysisChart
              data={ratingAnalysis.by_cfg}
              title={analysisTab === 'by-model' ? `Best CFG` : 'Best CFG Range'}
              color="#ec4899"
            />
          </div>
        </>
      )}
    </div>
  )
}
