import 'react'

// React 18 では camelCase の fetchPriority が未サポートのため、
// 小文字の fetchpriority 属性を型として追加する（React 19 移行時に削除可）
declare module 'react' {
  interface ImgHTMLAttributes<T> extends HTMLAttributes<T> {
    fetchpriority?: 'high' | 'low' | 'auto'
  }
}
