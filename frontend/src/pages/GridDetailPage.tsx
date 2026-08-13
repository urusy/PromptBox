import { useParams } from 'react-router-dom'
import DetailPage from './DetailPage'
import GridMembers from '@/components/grid/GridMembers'

/**
 * グリッド画像の詳細画面（/grids/:id）。
 *
 * 画像本体・メタデータ・評価は通常の詳細画面をそのまま使い（basePath を渡して
 * 前後移動をグリッド同士に留める）、その下にこの画面固有の「構成画像」を足す。
 * 詳細表示の機能を二重に持たないための構成。
 */
export default function GridDetailPage() {
  const { id } = useParams<{ id: string }>()

  return (
    <>
      <DetailPage basePath="/grids" listPath="/grids" />
      {id && <GridMembers imageId={id} />}
    </>
  )
}
