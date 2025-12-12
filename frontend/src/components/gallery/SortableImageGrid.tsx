import { useMemo } from 'react'
import {
  DndContext,
  closestCenter,
  KeyboardSensor,
  PointerSensor,
  TouchSensor,
  useSensor,
  useSensors,
  DragEndEvent,
  DragStartEvent,
  DragOverlay,
} from '@dnd-kit/core'
import {
  arrayMove,
  SortableContext,
  sortableKeyboardCoordinates,
  rectSortingStrategy,
  useSortable,
} from '@dnd-kit/sortable'
import { CSS } from '@dnd-kit/utilities'
import { GripVertical } from 'lucide-react'
import { useState } from 'react'
import type { ShowcaseImageInfo } from '@/types/showcase'

interface SortableImageGridProps {
  images: ShowcaseImageInfo[]
  onReorder: (newOrder: ShowcaseImageInfo[]) => void
}

interface SortableImageItemProps {
  image: ShowcaseImageInfo
}

function SortableImageItem({ image }: SortableImageItemProps) {
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({
    id: image.id,
    transition: {
      duration: 200,
      easing: 'ease',
    },
  })

  const style: React.CSSProperties = {
    transform: CSS.Translate.toString(transform),
    transition: transition || 'transform 200ms ease',
    // Use internal isDragging state from useSortable (more reliable)
    opacity: isDragging ? 0 : 1,
    visibility: isDragging ? 'hidden' as const : 'visible' as const,
    // Disable iOS magnifier/callout on long press
    WebkitTouchCallout: 'none',
    WebkitUserSelect: 'none',
    userSelect: 'none',
    touchAction: 'manipulation',
  }

  return (
    <div
      ref={setNodeRef}
      style={style}
      className="relative aspect-square rounded-lg overflow-hidden group bg-gray-800"
    >
      <img
        src={`/storage/${image.thumbnail_path}`}
        alt=""
        className="w-full h-full object-cover"
        loading="lazy"
        draggable={false}
      />
      {/* Drag handle overlay */}
      <div
        {...attributes}
        {...listeners}
        className="absolute inset-0 bg-black/0 hover:bg-black/30 transition-colors cursor-grab active:cursor-grabbing flex items-center justify-center"
      >
        <div className="opacity-0 group-hover:opacity-100 transition-opacity bg-black/60 rounded-lg p-2">
          <GripVertical size={24} className="text-white" />
        </div>
      </div>
    </div>
  )
}

function DragOverlayImage({ image }: { image: ShowcaseImageInfo }) {
  return (
    <div className="aspect-square rounded-lg overflow-hidden shadow-2xl ring-2 ring-blue-500 bg-gray-800">
      <img
        src={`/storage/${image.thumbnail_path}`}
        alt=""
        className="w-full h-full object-cover"
        draggable={false}
      />
    </div>
  )
}

export default function SortableImageGrid({ images, onReorder }: SortableImageGridProps) {
  const [activeId, setActiveId] = useState<string | null>(null)

  const sensors = useSensors(
    useSensor(PointerSensor, {
      activationConstraint: {
        distance: 8,
      },
    }),
    useSensor(TouchSensor, {
      activationConstraint: {
        delay: 200,
        tolerance: 5,
      },
    }),
    useSensor(KeyboardSensor, {
      coordinateGetter: sortableKeyboardCoordinates,
    })
  )

  const itemIds = useMemo(() => images.map((img) => img.id), [images])
  const activeImage = useMemo(
    () => images.find((img) => img.id === activeId),
    [images, activeId]
  )

  const handleDragStart = (event: DragStartEvent) => {
    setActiveId(event.active.id as string)
  }

  const handleDragEnd = (event: DragEndEvent) => {
    const { active, over } = event

    setActiveId(null)

    if (!over || active.id === over.id) return

    const oldIndex = images.findIndex((img) => img.id === active.id)
    const newIndex = images.findIndex((img) => img.id === over.id)

    if (oldIndex !== newIndex) {
      const newOrder = arrayMove(images, oldIndex, newIndex)
      onReorder(newOrder)
    }
  }

  const handleDragCancel = () => {
    setActiveId(null)
  }

  return (
    <DndContext
      sensors={sensors}
      collisionDetection={closestCenter}
      onDragStart={handleDragStart}
      onDragEnd={handleDragEnd}
      onDragCancel={handleDragCancel}
    >
      <SortableContext items={itemIds} strategy={rectSortingStrategy}>
        <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 2xl:grid-cols-7 3xl:grid-cols-8 4xl:grid-cols-10 gap-2">
          {images.map((image) => (
            <SortableImageItem
              key={image.id}
              image={image}
            />
          ))}
        </div>
      </SortableContext>

      <DragOverlay dropAnimation={null}>
        {activeImage ? <DragOverlayImage image={activeImage} /> : null}
      </DragOverlay>
    </DndContext>
  )
}
