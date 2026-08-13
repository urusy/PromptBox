import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { LayoutGrid, Info } from 'lucide-react'
import clsx from 'clsx'
import { gridMembersQueryOptions } from '@/api/grids'
import type { GridAxes, GridConfidence, GridMember } from '@/types/grid'
import ImageCard from '@/components/gallery/ImageCard'

interface GridMembersProps {
  imageId: string
}

const WINDOW_OPTIONS = [1, 6, 24, 72, 168] as const

/** 構成画像を1画面に収めるための列数の上限。 */
const MAX_COLUMNS = 8

const CONFIDENCE_STYLE: Record<GridConfidence, { label: string; className: string }> = {
  exact: { label: '一致', className: 'bg-green-900/60 text-green-300 border-green-700' },
  partial: { label: '一部', className: 'bg-yellow-900/60 text-yellow-300 border-yellow-700' },
  heuristic: { label: '推定', className: 'bg-orange-900/60 text-orange-300 border-orange-700' },
  none: { label: '特定不可', className: 'bg-gray-700 text-gray-300 border-gray-600' },
}

const CONFIDENCE_HELP: Record<GridConfidence, string> = {
  exact: '軸から期待されるセル数と一致しました。',
  partial: '期待されるセル数と一致しません。削除済みか、取り込まれていない画像があります。',
  heuristic: '軸の種類が未対応のため、共通する生成パラメータと生成時刻だけで絞り込みました。',
  none: 'この画像には軸のメタデータが無いため、構成画像を特定できません。',
}

/** 軸の値からセルの見出しを作る（例: "CFG 7 / Euler a"）。 */
function cellLabel(member: GridMember, axes: GridAxes | null): string {
  if (!axes) return ''
  const parts: string[] = []
  if (axes.x && member.axis_values.x) parts.push(`${axes.x.type}: ${member.axis_values.x}`)
  if (axes.y && member.axis_values.y) parts.push(`${axes.y.type}: ${member.axis_values.y}`)
  if (axes.z && member.axis_values.z) parts.push(`${axes.z.type}: ${member.axis_values.z}`)
  return parts.join(' / ')
}

function AxisTable({ axes }: { axes: GridAxes }) {
  const rows = (['x', 'y', 'z'] as const)
    .map((key) => ({ key, axis: axes[key] }))
    .filter((row) => row.axis !== null)

  if (rows.length === 0) return null

  return (
    <div className="overflow-x-auto">
      <table className="w-full text-sm">
        <thead>
          <tr className="text-left text-gray-400 border-b border-gray-700">
            <th className="py-2 pr-4 font-medium w-8">軸</th>
            <th className="py-2 pr-4 font-medium">種類</th>
            <th className="py-2 font-medium">値</th>
          </tr>
        </thead>
        <tbody>
          {rows.map(({ key, axis }) => (
            <tr key={key} className="border-b border-gray-800 last:border-0">
              <td className="py-2 pr-4 text-gray-500 uppercase">{key}</td>
              <td className="py-2 pr-4 text-gray-300">
                {axis!.type}
                {axis!.column === null && (
                  <span className="ml-2 text-xs text-orange-400">(未対応)</span>
                )}
              </td>
              <td className="py-2 text-gray-300">
                <div className="flex flex-wrap gap-1">
                  {axis!.values.map((value) => (
                    <span key={value} className="px-2 py-0.5 bg-gray-700 rounded text-xs">
                      {value}
                    </span>
                  ))}
                </div>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  )
}

/**
 * グリッド詳細の「構成画像」セクション。
 *
 * グリッドとセルを結ぶ情報はどこにも保存されていないため、バックエンドが
 * 軸の値・共通パラメータ・生成時刻から推定した結果を表示する。だから
 * confidence と対象期間を常に見せ、期間はその場で変えられるようにしている。
 */
export default function GridMembers({ imageId }: GridMembersProps) {
  const [windowHours, setWindowHours] = useState<number | undefined>(undefined)
  const { data, isLoading, error } = useQuery(gridMembersQueryOptions(imageId, windowHours))

  // グリッドではない画像（400）では、セクションごと出さない。
  if (error) return null

  if (isLoading) {
    return (
      <div className="mt-8 bg-gray-800 rounded-lg p-6">
        <div className="h-6 w-40 bg-gray-700 rounded animate-pulse" />
      </div>
    )
  }

  if (!data) return null

  const confidence = CONFIDENCE_STYLE[data.confidence]
  const columns = Math.min(data.axes?.x?.values.length || 4, MAX_COLUMNS)

  return (
    <div className="mt-8 bg-gray-800 rounded-lg p-4 sm:p-6">
      <div className="flex flex-wrap items-center justify-between gap-3 mb-4">
        <div className="flex items-center gap-3">
          <h2 className="text-lg font-semibold flex items-center gap-2">
            <LayoutGrid size={20} className="text-blue-400" />
            構成画像
          </h2>
          <span
            className={clsx('px-2 py-0.5 text-xs rounded border', confidence.className)}
            title={CONFIDENCE_HELP[data.confidence]}
          >
            {confidence.label}
          </span>
          <span className="text-sm text-gray-400">
            {data.matched}
            {data.expected_cells !== null && ` / ${data.expected_cells}`} 枚
          </span>
        </div>

        <label className="flex items-center gap-2 text-sm text-gray-400">
          対象期間
          <select
            value={data.window_hours}
            onChange={(e) => setWindowHours(Number(e.target.value))}
            className="bg-gray-700 text-gray-200 rounded-lg px-2 py-1 border-none focus:ring-1 focus:ring-blue-500 cursor-pointer"
            title="グリッド保存時刻から何時間さかのぼって構成画像を探すか"
          >
            {WINDOW_OPTIONS.map((hours) => (
              <option key={hours} value={hours}>
                {hours < 24 ? `${hours}時間` : `${hours / 24}日`}
              </option>
            ))}
          </select>
        </label>
      </div>

      {data.axes && (
        <div className="mb-4">
          <AxisTable axes={data.axes} />
        </div>
      )}

      {data.confidence !== 'exact' && (
        <p className="flex items-start gap-2 text-sm text-gray-400 mb-4">
          <Info size={16} className="mt-0.5 shrink-0" />
          <span>{CONFIDENCE_HELP[data.confidence]}</span>
        </p>
      )}

      {data.members.length > 0 ? (
        <div
          className="grid gap-3"
          style={{ gridTemplateColumns: `repeat(${columns}, minmax(0, 1fr))` }}
        >
          {data.members.map((member) => {
            const label = cellLabel(member, data.axes)
            return (
              <div key={member.id} className="min-w-0">
                {/* 構成画像は通常の詳細画面へ遷移する（グリッド詳細ではない）。 */}
                <ImageCard
                  image={member}
                  aspectRatio={
                    member.width && member.height ? member.width / member.height : undefined
                  }
                />
                {label && (
                  <p className="mt-1 text-xs text-gray-400 truncate" title={label}>
                    {label}
                  </p>
                )}
              </div>
            )
          })}
        </div>
      ) : (
        <p className="text-gray-500 py-4">構成画像は見つかりませんでした。</p>
      )}
    </div>
  )
}
