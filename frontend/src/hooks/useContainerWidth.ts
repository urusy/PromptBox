import { useState, useEffect, useRef } from 'react'
import type { RefObject } from 'react'

/**
 * ResizeObserver で対象要素の実測幅(px)を追跡するフック。
 * Justified レイアウトのように、padding を含まないコンテナ実幅が必要な場面で使う。
 *
 * @returns [ref, width] ref を計測対象要素に付与すると width が実測幅に更新される
 */
export function useContainerWidth<T extends HTMLElement = HTMLDivElement>(): [
  RefObject<T>,
  number,
] {
  const ref = useRef<T>(null)
  const [width, setWidth] = useState(0)

  useEffect(() => {
    const element = ref.current
    if (!element) return

    // 初期幅を即時反映
    setWidth(element.clientWidth)

    const observer = new ResizeObserver((entries) => {
      for (const entry of entries) {
        const next = entry.contentRect.width
        // 幅が変わったときのみ更新（不要な再レンダリングを避ける）
        setWidth((prev) => (Math.abs(prev - next) > 0.5 ? next : prev))
      }
    })

    observer.observe(element)
    return () => observer.disconnect()
  }, [])

  return [ref, width]
}
