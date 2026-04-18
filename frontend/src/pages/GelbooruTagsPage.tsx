import { useState, useEffect, useMemo, useCallback, useRef } from 'react'
import { useQuery } from '@tanstack/react-query'
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
  verticalListSortingStrategy,
  useSortable,
} from '@dnd-kit/sortable'
import { CSS } from '@dnd-kit/utilities'
import { Search, Tag, X, Copy, GripVertical, ChevronUp, ChevronDown, Pencil } from 'lucide-react'
import toast from 'react-hot-toast'
import { gelbooruApi } from '@/api/gelbooru'
import { TAG_TYPE_COLORS } from '@/types/gelbooru'

function formatCount(count: number): string {
  if (count >= 1_000_000) return `${(count / 1_000_000).toFixed(1)}M`
  if (count >= 1_000) return `${(count / 1_000).toFixed(1)}k`
  return String(count)
}

function getTagColor(type: number) {
  return TAG_TYPE_COLORS[type] || TAG_TYPE_COLORS[0]
}

/** Sortable chip for the prompt builder */
function SortableTagChip({
  tag,
  index,
  total,
  onRemove,
  onMoveUp,
  onMoveDown,
}: {
  tag: string
  index: number
  total: number
  onRemove: () => void
  onMoveUp: () => void
  onMoveDown: () => void
}) {
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({
    id: tag,
    transition: { duration: 200, easing: 'ease' },
  })

  const style: React.CSSProperties = {
    transform: CSS.Translate.toString(transform),
    transition: transition || 'transform 200ms ease',
    opacity: isDragging ? 0 : 1,
    WebkitTouchCallout: 'none',
    WebkitUserSelect: 'none',
    userSelect: 'none',
  }

  return (
    <div
      ref={setNodeRef}
      style={style}
      className="flex items-center gap-1 bg-gray-700 rounded-lg px-2 py-1 group"
    >
      <div
        {...attributes}
        {...listeners}
        className="cursor-grab active:cursor-grabbing text-gray-500 hover:text-gray-300 touch-none"
      >
        <GripVertical size={14} />
      </div>
      <span className="text-sm text-gray-200">{tag}</span>
      <div className="flex items-center gap-0.5 ml-1 opacity-0 group-hover:opacity-100 transition-opacity">
        <button
          onClick={onMoveUp}
          disabled={index === 0}
          className="p-0.5 hover:bg-gray-600 rounded disabled:opacity-30"
          title="Move up"
        >
          <ChevronUp size={12} />
        </button>
        <button
          onClick={onMoveDown}
          disabled={index === total - 1}
          className="p-0.5 hover:bg-gray-600 rounded disabled:opacity-30"
          title="Move down"
        >
          <ChevronDown size={12} />
        </button>
      </div>
      <button
        onClick={onRemove}
        className="p-0.5 text-gray-500 hover:text-red-400 hover:bg-gray-600 rounded transition-colors"
        title="Remove"
      >
        <X size={12} />
      </button>
    </div>
  )
}

function DragOverlayChip({ tag }: { tag: string }) {
  return (
    <div className="flex items-center gap-1 bg-gray-600 rounded-lg px-2 py-1 shadow-xl ring-2 ring-blue-500">
      <GripVertical size={14} className="text-gray-400" />
      <span className="text-sm text-gray-200">{tag}</span>
    </div>
  )
}

export default function GelbooruTagsPage() {
  const [searchQuery, setSearchQuery] = useState('')
  const [debouncedQuery, setDebouncedQuery] = useState('')
  const [selectedTags, setSelectedTags] = useState<string[]>([])
  const [isEditMode, setIsEditMode] = useState(false)
  const [editText, setEditText] = useState('')
  const [activeId, setActiveId] = useState<string | null>(null)
  const textareaRef = useRef<HTMLTextAreaElement>(null)

  // Debounce search query
  useEffect(() => {
    const timer = setTimeout(() => {
      setDebouncedQuery(searchQuery.trim())
    }, 300)
    return () => clearTimeout(timer)
  }, [searchQuery])

  const { data, isLoading, isFetching } = useQuery({
    queryKey: ['gelbooru-tags', debouncedQuery],
    queryFn: () => gelbooruApi.searchTags(debouncedQuery),
    enabled: debouncedQuery.length >= 2,
    staleTime: 5 * 60 * 1000, // 5 minutes
  })

  const tags = data?.tags ?? []

  // DnD sensors
  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 8 } }),
    useSensor(TouchSensor, { activationConstraint: { delay: 200, tolerance: 5 } }),
    useSensor(KeyboardSensor, { coordinateGetter: sortableKeyboardCoordinates })
  )

  const activeTag = useMemo(
    () => (activeId ? selectedTags.find((t) => t === activeId) : undefined),
    [selectedTags, activeId]
  )

  const addTag = useCallback(
    (tagName: string) => {
      const normalized = tagName.replace(/ /g, '_')
      if (!selectedTags.includes(normalized)) {
        setSelectedTags((prev) => [...prev, normalized])
      }
    },
    [selectedTags]
  )

  const removeTag = useCallback((tagName: string) => {
    setSelectedTags((prev) => prev.filter((t) => t !== tagName))
  }, [])

  const moveTag = useCallback((index: number, direction: -1 | 1) => {
    setSelectedTags((prev) => {
      const newIndex = index + direction
      if (newIndex < 0 || newIndex >= prev.length) return prev
      return arrayMove(prev, index, newIndex)
    })
  }, [])

  const handleDragStart = (event: DragStartEvent) => {
    setActiveId(event.active.id as string)
  }

  const handleDragEnd = (event: DragEndEvent) => {
    const { active, over } = event
    setActiveId(null)
    if (!over || active.id === over.id) return

    setSelectedTags((prev) => {
      const oldIndex = prev.indexOf(active.id as string)
      const newIndex = prev.indexOf(over.id as string)
      if (oldIndex === -1 || newIndex === -1) return prev
      return arrayMove(prev, oldIndex, newIndex)
    })
  }

  const handleCopy = async () => {
    const text = selectedTags.join(', ')
    try {
      await navigator.clipboard.writeText(text)
      toast.success('Copied to clipboard')
    } catch {
      // Fallback for Safari / non-HTTPS
      const textarea = document.createElement('textarea')
      textarea.value = text
      document.body.appendChild(textarea)
      textarea.select()
      document.execCommand('copy')
      document.body.removeChild(textarea)
      toast.success('Copied to clipboard')
    }
  }

  const enterEditMode = () => {
    setEditText(selectedTags.join(', '))
    setIsEditMode(true)
    setTimeout(() => textareaRef.current?.focus(), 0)
  }

  const exitEditMode = () => {
    const parsed = editText
      .split(',')
      .map((t) => t.trim().replace(/ /g, '_'))
      .filter((t) => t.length > 0)
    // Deduplicate while preserving order
    const unique = [...new Set(parsed)]
    setSelectedTags(unique)
    setIsEditMode(false)
  }

  const clearAll = () => {
    setSelectedTags([])
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center gap-3">
        <Tag size={28} className="text-teal-400" />
        <h1 className="text-2xl font-bold">Gelbooru Tags</h1>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Left: Search */}
        <div className="space-y-4">
          <div className="bg-gray-800 rounded-lg p-4 space-y-4">
            <h2 className="text-lg font-semibold">Tag Search</h2>

            {/* Search input */}
            <div className="relative">
              <Search
                size={18}
                className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400"
              />
              <input
                type="text"
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                placeholder="Search tags (min 2 characters)..."
                className="w-full pl-10 pr-4 py-2 bg-gray-700 border border-gray-600 rounded-lg focus:outline-none focus:ring-2 focus:ring-teal-500"
              />
              {searchQuery && (
                <button
                  onClick={() => setSearchQuery('')}
                  className="absolute right-3 top-1/2 -translate-y-1/2 text-gray-400 hover:text-gray-200"
                >
                  <X size={16} />
                </button>
              )}
            </div>

            {/* Loading indicator */}
            {(isLoading || isFetching) && debouncedQuery.length >= 2 && (
              <div className="text-sm text-gray-400">Searching...</div>
            )}

            {/* Results */}
            {tags.length > 0 && (
              <div className="space-y-1 max-h-[60vh] overflow-y-auto">
                {tags.map((tag) => {
                  const color = getTagColor(tag.type)
                  const isSelected = selectedTags.includes(tag.name)
                  return (
                    <button
                      key={tag.id}
                      onClick={() => addTag(tag.name)}
                      disabled={isSelected}
                      className={`w-full flex items-center justify-between px-3 py-2 rounded-lg text-left transition-colors ${
                        isSelected
                          ? 'bg-gray-700/50 opacity-50 cursor-not-allowed'
                          : 'hover:bg-gray-700 cursor-pointer'
                      }`}
                    >
                      <div className="flex items-center gap-2 min-w-0">
                        <span
                          className={`shrink-0 px-1.5 py-0.5 rounded text-xs ${color.bg} ${color.text}`}
                        >
                          {color.label}
                        </span>
                        <span className="text-sm text-gray-200 truncate">{tag.name}</span>
                      </div>
                      <span className="shrink-0 text-xs text-gray-500 ml-2">
                        {formatCount(tag.count)}
                      </span>
                    </button>
                  )
                })}
              </div>
            )}

            {/* No results */}
            {debouncedQuery.length >= 2 && !isLoading && !isFetching && tags.length === 0 && (
              <div className="text-sm text-gray-500 text-center py-4">No tags found</div>
            )}

            {/* Hint */}
            {debouncedQuery.length < 2 && (
              <div className="text-sm text-gray-500 text-center py-4">
                Type at least 2 characters to search
              </div>
            )}
          </div>
        </div>

        {/* Right: Prompt Builder */}
        <div className="space-y-4">
          <div className="bg-gray-800 rounded-lg p-4 space-y-4">
            <div className="flex items-center justify-between">
              <h2 className="text-lg font-semibold">
                Prompt Builder
                {selectedTags.length > 0 && (
                  <span className="text-sm text-gray-400 font-normal ml-2">
                    ({selectedTags.length} tags)
                  </span>
                )}
              </h2>
              <div className="flex items-center gap-2">
                {selectedTags.length > 0 && (
                  <>
                    <button
                      onClick={enterEditMode}
                      className="flex items-center gap-1 px-3 py-1.5 text-sm bg-gray-700 hover:bg-gray-600 rounded-lg transition-colors"
                      title="Edit as text"
                    >
                      <Pencil size={14} />
                      <span className="hidden sm:inline">Edit</span>
                    </button>
                    <button
                      onClick={handleCopy}
                      className="flex items-center gap-1 px-3 py-1.5 text-sm bg-teal-600 hover:bg-teal-700 rounded-lg transition-colors"
                      title="Copy to clipboard"
                    >
                      <Copy size={14} />
                      <span className="hidden sm:inline">Copy</span>
                    </button>
                    <button
                      onClick={clearAll}
                      className="flex items-center gap-1 px-3 py-1.5 text-sm bg-gray-700 hover:bg-red-600/80 rounded-lg transition-colors text-gray-400 hover:text-white"
                      title="Clear all"
                    >
                      <X size={14} />
                      <span className="hidden sm:inline">Clear</span>
                    </button>
                  </>
                )}
              </div>
            </div>

            {/* Edit mode: textarea */}
            {isEditMode ? (
              <div className="space-y-2">
                <textarea
                  ref={textareaRef}
                  value={editText}
                  onChange={(e) => setEditText(e.target.value)}
                  onBlur={exitEditMode}
                  onKeyDown={(e) => {
                    if (e.key === 'Escape') exitEditMode()
                  }}
                  className="w-full h-40 px-3 py-2 bg-gray-700 border border-teal-500 rounded-lg text-sm text-gray-200 focus:outline-none focus:ring-2 focus:ring-teal-500 resize-y"
                  placeholder="Enter tags separated by commas..."
                />
                <p className="text-xs text-gray-500">
                  Comma-separated tags. Click outside or press Escape to apply.
                </p>
              </div>
            ) : (
              <>
                {/* Tag chips with DnD */}
                {selectedTags.length > 0 ? (
                  <DndContext
                    sensors={sensors}
                    collisionDetection={closestCenter}
                    onDragStart={handleDragStart}
                    onDragEnd={handleDragEnd}
                    onDragCancel={() => setActiveId(null)}
                  >
                    <SortableContext
                      items={selectedTags}
                      strategy={verticalListSortingStrategy}
                    >
                      <div className="flex flex-wrap gap-2">
                        {selectedTags.map((tag, index) => (
                          <SortableTagChip
                            key={tag}
                            tag={tag}
                            index={index}
                            total={selectedTags.length}
                            onRemove={() => removeTag(tag)}
                            onMoveUp={() => moveTag(index, -1)}
                            onMoveDown={() => moveTag(index, 1)}
                          />
                        ))}
                      </div>
                    </SortableContext>
                    <DragOverlay dropAnimation={null}>
                      {activeTag ? <DragOverlayChip tag={activeTag} /> : null}
                    </DragOverlay>
                  </DndContext>
                ) : (
                  <div className="text-sm text-gray-500 text-center py-8">
                    Click tags from search results to add them here
                  </div>
                )}

                {/* Preview */}
                {selectedTags.length > 0 && (
                  <div className="mt-4 p-3 bg-gray-900 rounded-lg">
                    <div className="text-xs text-gray-500 mb-1">Preview</div>
                    <p className="text-sm text-gray-300 break-all">{selectedTags.join(', ')}</p>
                  </div>
                )}
              </>
            )}
          </div>
        </div>
      </div>
    </div>
  )
}
