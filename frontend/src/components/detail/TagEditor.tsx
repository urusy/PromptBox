import { useState, useRef, useEffect } from 'react'
import { X, Plus } from 'lucide-react'

interface TagEditorProps {
  tags: string[]
  onChange: (tags: string[]) => void
}

export default function TagEditor({ tags, onChange }: TagEditorProps) {
  const [inputValue, setInputValue] = useState('')
  const [isAdding, setIsAdding] = useState(false)
  const inputRef = useRef<HTMLInputElement>(null)

  useEffect(() => {
    if (isAdding && inputRef.current) {
      inputRef.current.focus()
    }
  }, [isAdding])

  const handleAddTag = () => {
    const newTag = inputValue.trim().toLowerCase()
    if (newTag && !tags.includes(newTag)) {
      onChange([...tags, newTag])
    }
    setInputValue('')
    setIsAdding(false)
  }

  const handleRemoveTag = (tagToRemove: string) => {
    onChange(tags.filter((tag) => tag !== tagToRemove))
  }

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      e.preventDefault()
      handleAddTag()
    } else if (e.key === 'Escape') {
      setInputValue('')
      setIsAdding(false)
    }
  }

  return (
    <div>
      <h3 className="text-lg font-medium text-white mb-2">Tags</h3>
      <div className="flex flex-wrap gap-2">
        {tags.map((tag) => (
          <span
            key={tag}
            className="flex items-center gap-1 px-2 py-1 bg-blue-600/20 text-blue-400 text-sm rounded group"
          >
            {tag}
            <button
              onClick={() => handleRemoveTag(tag)}
              className="opacity-0 group-hover:opacity-100 transition-opacity hover:text-red-400"
            >
              <X size={14} />
            </button>
          </span>
        ))}

        {isAdding ? (
          <input
            ref={inputRef}
            type="text"
            value={inputValue}
            onChange={(e) => setInputValue(e.target.value)}
            onBlur={handleAddTag}
            onKeyDown={handleKeyDown}
            placeholder="Enter tag..."
            className="px-2 py-1 bg-gray-700 border border-gray-600 rounded text-sm text-white placeholder-gray-400 focus:outline-none focus:ring-1 focus:ring-blue-500 w-24"
          />
        ) : (
          <button
            onClick={() => setIsAdding(true)}
            className="flex items-center gap-1 px-2 py-1 bg-gray-700 hover:bg-gray-600 text-gray-400 hover:text-white text-sm rounded transition-colors"
          >
            <Plus size={14} />
            Add
          </button>
        )}
      </div>
    </div>
  )
}
