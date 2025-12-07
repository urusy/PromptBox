import type { ImageListItem } from '@/types/image'
import ImageCard from './ImageCard'

interface ImageGridProps {
  images: ImageListItem[]
}

export default function ImageGrid({ images }: ImageGridProps) {
  if (images.length === 0) {
    return (
      <div className="text-center text-gray-400 py-16">
        <p className="text-xl">No images found</p>
        <p className="mt-2">Try adjusting your search filters</p>
      </div>
    )
  }

  return (
    <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 2xl:grid-cols-8 gap-3">
      {images.map((image) => (
        <ImageCard key={image.id} image={image} />
      ))}
    </div>
  )
}
