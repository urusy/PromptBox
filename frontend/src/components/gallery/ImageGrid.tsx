import type { ImageListItem } from '@/types/image'
import ImageCard from './ImageCard'

export type GridSize = 'small' | 'medium' | 'large'

interface ImageGridProps {
  images: ImageListItem[]
  size?: GridSize
}

const GRID_CLASSES: Record<GridSize, string> = {
  small: 'grid-cols-3 sm:grid-cols-4 md:grid-cols-5 lg:grid-cols-6 xl:grid-cols-8 2xl:grid-cols-10',
  medium: 'grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 2xl:grid-cols-8',
  large: 'grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-4 2xl:grid-cols-5',
}

export default function ImageGrid({ images, size = 'medium' }: ImageGridProps) {
  if (images.length === 0) {
    return (
      <div className="text-center text-gray-400 py-16">
        <p className="text-xl">No images found</p>
        <p className="mt-2">Try adjusting your search filters</p>
      </div>
    )
  }

  return (
    <div className={`grid ${GRID_CLASSES[size]} gap-3`}>
      {images.map((image) => (
        <ImageCard key={image.id} image={image} />
      ))}
    </div>
  )
}
