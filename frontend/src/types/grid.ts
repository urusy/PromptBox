import type { ImageListItem } from './image'

/** グリッドの1軸。column はバックエンドが対応している場合のみ入る。 */
export interface GridAxis {
  type: string
  values: string[]
  column: string | null
}

export interface GridAxes {
  x: GridAxis | null
  y: GridAxis | null
  z: GridAxis | null
}

export interface GridPosition {
  x: number
  y: number
  z: number
}

export interface GridAxisValues {
  x: string | null
  y: string | null
  z: string | null
}

/** 構成画像。一覧アイテムに、グリッド上の位置と軸の値が付く。 */
export interface GridMember extends ImageListItem {
  position: GridPosition
  axis_values: GridAxisValues
}

/**
 * 構成画像の確からしさ。グリッドと構成画像を結ぶ情報は保存されていないため、
 * 推定結果であることを表に出す。
 * - exact: 軸から期待されるセル数と一致
 * - partial: 数が合わない（削除済み・取り込み漏れなど）
 * - heuristic: 軸の種類が未対応で、共通パラメータと時間だけで絞った
 * - none: 軸のメタデータが無く、特定できない
 */
export type GridConfidence = 'exact' | 'partial' | 'heuristic' | 'none'

export interface ApiWarning {
  code: string
  param: string
  message: string
  hint?: string
}

export interface GridMembersResponse {
  grid: ImageListItem
  axes: GridAxes | null
  members: GridMember[]
  expected_cells: number | null
  matched: number
  confidence: GridConfidence
  window_hours: number
  warnings?: ApiWarning[]
}
