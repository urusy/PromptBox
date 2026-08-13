import type { ImageListItem } from '@/types/image'

/**
 * 画像のアスペクト比（幅/高さ）を返す。
 * width/height が欠損・0 の場合は正方形(1)にフォールバックする。
 */
export function getAspectRatio(image: Pick<ImageListItem, 'width' | 'height'>): number {
  return image.width && image.height ? image.width / image.height : 1
}

/**
 * バランス型 Masonry（Waterfall）の列振り分け。
 * 各列の累積高さ（1/aspectRatio を加算＝列幅を1と正規化した相対高さ）を追跡し、
 * 常に最も背の低い列へ次の画像を追加する。
 */
export function distributeWaterfall(
  images: ImageListItem[],
  columnCount: number
): ImageListItem[][] {
  const count = Math.max(1, columnCount)
  const columns: ImageListItem[][] = Array.from({ length: count }, () => [])
  const heights = new Array<number>(count).fill(0)

  for (const image of images) {
    let shortest = 0
    for (let i = 1; i < count; i++) {
      if (heights[i] < heights[shortest]) shortest = i
    }
    columns[shortest].push(image)
    heights[shortest] += 1 / getAspectRatio(image)
  }

  return columns
}

export interface JustifiedItem {
  image: ImageListItem
  /** この画像の表示幅(px)。高さは行の rowHeight に一致する。 */
  width: number
}

export interface JustifiedRow {
  items: JustifiedItem[]
  /** 行の共通高さ(px)。 */
  rowHeight: number
}

/**
 * Justified rows（Google Photos 風の行揃え）を計算する。
 * 目標行高で各画像の幅を積算し、コンテナ幅を超えたら行を確定。
 * 行内の合計アスペクト比からコンテナ幅いっぱいに収まる実行高を逆算する。
 * 最終行は引き伸ばしすぎないよう targetRowHeight*1.5 で高さをクランプする。
 */
export function computeJustifiedRows(
  images: ImageListItem[],
  containerWidth: number,
  targetRowHeight: number,
  gap: number
): JustifiedRow[] {
  const rows: JustifiedRow[] = []
  if (containerWidth <= 0 || images.length === 0) return rows

  const maxRowHeight = targetRowHeight * 1.5

  let rowImages: ImageListItem[] = []
  let rowAspectSum = 0

  const flushRow = (isLastRow: boolean) => {
    if (rowImages.length === 0) return
    const n = rowImages.length
    const gapTotal = gap * (n - 1)
    const availableWidth = containerWidth - gapTotal
    // 行を横幅いっぱいに揃えるための実行高。最終行のみクランプして巨大化を防ぐ。
    let rowHeight = availableWidth / rowAspectSum
    if (isLastRow) rowHeight = Math.min(rowHeight, maxRowHeight)

    const items: JustifiedItem[] = rowImages.map((image) => ({
      image,
      width: rowHeight * getAspectRatio(image),
    }))
    rows.push({ items, rowHeight })

    rowImages = []
    rowAspectSum = 0
  }

  for (const image of images) {
    const aspect = getAspectRatio(image)
    rowImages.push(image)
    rowAspectSum += aspect

    // 目標行高で並べたときの合計幅がコンテナ幅を超えたら行を確定
    const projectedWidth =
      rowAspectSum * targetRowHeight + gap * (rowImages.length - 1)
    if (projectedWidth >= containerWidth) {
      flushRow(false)
    }
  }

  // 余った最終行
  flushRow(true)

  return rows
}
