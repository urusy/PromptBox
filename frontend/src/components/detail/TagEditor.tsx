import { useState, useRef, useEffect } from 'react'
import { useQuery } from '@tanstack/react-query'
import { X, Plus } from 'lucide-react'
import { tagsApi } from '@/api/tags'

interface TagEditorProps {
  tags: string[]
  onChange: (tags: string[]) => void
}

export default function TagEditor({ tags, onChange }: TagEditorProps) {
  const [inputValue, setInputValue] = useState('')
  const [isAdding, setIsAdding] = useState(false)
  const [showSuggestions, setShowSuggestions] = useState(false)
  const [selectedIndex, setSelectedIndex] = useState(-1)
  const inputRef = useRef<HTMLInputElement>(null)
  const suggestionsRef = useRef<HTMLDivElement>(null)

  // Fetch tags - without query: recent 10, with query: search all tags
  const { data: suggestedTags = [] } = useQuery({
    queryKey: ['tags', inputValue],
    queryFn: () => tagsApi.list(inputValue || undefined, inputValue ? 20 : 10),
    staleTime: inputValue ? 0 : 30000, // Don't cache search results
  })

  // Exclude already added tags
  const filteredSuggestions = suggestedTags.filter((tag) => !tags.includes(tag))

  useEffect(() => {
    if (isAdding && inputRef.current) {
      inputRef.current.focus()
    }
  }, [isAdding])

  // Reset selected index when suggestions change
  useEffect(() => {
    setSelectedIndex(-1)
  }, [filteredSuggestions.length, inputValue])

  // Close suggestions when clicking outside
  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (
        suggestionsRef.current &&
        !suggestionsRef.current.contains(e.target as Node) &&
        inputRef.current &&
        !inputRef.current.contains(e.target as Node)
      ) {
        setShowSuggestions(false)
      }
    }

    document.addEventListener('mousedown', handleClickOutside)
    return () => document.removeEventListener('mousedown', handleClickOutside)
  }, [])

  const handleAddTag = (tagToAdd?: string) => {
    const newTag = (tagToAdd || inputValue).trim().toLowerCase()
    if (newTag && !tags.includes(newTag)) {
      onChange([...tags, newTag])
    }
    setInputValue('')
    setShowSuggestions(false)
    setIsAdding(false)
  }

  const handleRemoveTag = (tagToRemove: string) => {
    onChange(tags.filter((tag) => tag !== tagToRemove))
  }

  const handleSelectSuggestion = (tag: string) => {
    handleAddTag(tag)
  }

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      e.preventDefault()
      if (selectedIndex >= 0 && selectedIndex < filteredSuggestions.length) {
        handleSelectSuggestion(filteredSuggestions[selectedIndex])
      } else {
        handleAddTag()
      }
    } else if (e.key === 'Escape') {
      setInputValue('')
      setShowSuggestions(false)
      setIsAdding(false)
    } else if (e.key === 'ArrowDown') {
      e.preventDefault()
      if (filteredSuggestions.length > 0) {
        setSelectedIndex((prev) =>
          prev < filteredSuggestions.length - 1 ? prev + 1 : prev
        )
      }
    } else if (e.key === 'ArrowUp') {
      e.preventDefault()
      if (filteredSuggestions.length > 0) {
        setSelectedIndex((prev) => (prev > 0 ? prev - 1 : -1))
      }
    }
  }

  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setInputValue(e.target.value)
    setShowSuggestions(true)
  }

  const handleInputFocus = () => {
    setShowSuggestions(true)
  }

  const handleInputBlur = () => {
    // Delay to allow click on suggestions
    setTimeout(() => {
      if (inputValue.trim()) {
        handleAddTag()
      } else {
        setIsAdding(false)
      }
      setShowSuggestions(false)
    }, 150)
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
          <div className="relative">
            <input
              ref={inputRef}
              type="text"
              value={inputValue}
              onChange={handleInputChange}
              onFocus={handleInputFocus}
              onBlur={handleInputBlur}
              onKeyDown={handleKeyDown}
              placeholder="Enter tag..."
              aria-label="タグを入力"
              aria-expanded={showSuggestions && filteredSuggestions.length > 0}
              aria-haspopup="listbox"
              aria-autocomplete="list"
              className="px-2 py-1 bg-gray-700 border border-gray-600 rounded text-sm text-white placeholder-gray-400 focus:outline-none focus:ring-1 focus:ring-blue-500 w-32"
            />

            {/* Tag suggestions dropdown */}
            {showSuggestions && filteredSuggestions.length > 0 && (
              <div
                ref={suggestionsRef}
                role="listbox"
                aria-label="タグ候補"
                className="absolute top-full left-0 mt-1 w-48 bg-gray-800 border border-gray-600 rounded shadow-lg z-10 max-h-48 overflow-y-auto"
              >
                <div className="px-2 py-1 text-xs text-gray-500 border-b border-gray-700">
                  {inputValue ? '検索結果' : '最近使用したタグ'}
                </div>
                {filteredSuggestions.map((tag, index) => (
                  <button
                    key={tag}
                    type="button"
                    role="option"
                    aria-selected={index === selectedIndex}
                    onMouseDown={(e) => {
                      e.preventDefault()
                      handleSelectSuggestion(tag)
                    }}
                    className={`w-full text-left px-2 py-1.5 text-sm transition-colors ${
                      index === selectedIndex
                        ? 'bg-blue-600 text-white'
                        : 'text-gray-300 hover:bg-gray-700'
                    }`}
                  >
                    {tag}
                  </button>
                ))}
              </div>
            )}
          </div>
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
