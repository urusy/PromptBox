import { useState, useEffect } from 'react'
import { Edit2, Check, X } from 'lucide-react'

interface MemoEditorProps {
  memo: string | null
  onChange: (memo: string | null) => void
}

export default function MemoEditor({ memo, onChange }: MemoEditorProps) {
  const [isEditing, setIsEditing] = useState(false)
  const [editValue, setEditValue] = useState(memo || '')

  useEffect(() => {
    setEditValue(memo || '')
  }, [memo])

  const handleSave = () => {
    const newMemo = editValue.trim() || null
    onChange(newMemo)
    setIsEditing(false)
  }

  const handleCancel = () => {
    setEditValue(memo || '')
    setIsEditing(false)
  }

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Escape') {
      handleCancel()
    } else if (e.key === 'Enter' && e.ctrlKey) {
      handleSave()
    }
  }

  return (
    <div>
      <div className="flex items-center justify-between mb-2">
        <h3 className="text-lg font-medium text-white">Memo</h3>
        {!isEditing && (
          <button
            onClick={() => setIsEditing(true)}
            className="text-gray-400 hover:text-white p-1"
            title="Edit memo"
          >
            <Edit2 size={16} />
          </button>
        )}
      </div>

      {isEditing ? (
        <div>
          <textarea
            value={editValue}
            onChange={(e) => setEditValue(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Add a memo..."
            rows={3}
            className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500 resize-none"
            autoFocus
          />
          <div className="flex items-center gap-2 mt-2">
            <button
              onClick={handleSave}
              className="flex items-center gap-1 px-3 py-1 bg-blue-600 hover:bg-blue-700 text-white text-sm rounded transition-colors"
            >
              <Check size={14} />
              Save
            </button>
            <button
              onClick={handleCancel}
              className="flex items-center gap-1 px-3 py-1 bg-gray-700 hover:bg-gray-600 text-gray-300 text-sm rounded transition-colors"
            >
              <X size={14} />
              Cancel
            </button>
            <span className="text-xs text-gray-500">Ctrl+Enter to save</span>
          </div>
        </div>
      ) : memo ? (
        <p className="text-sm text-gray-300 bg-gray-800 p-3 rounded-lg whitespace-pre-wrap">
          {memo}
        </p>
      ) : (
        <p className="text-sm text-gray-500 italic">No memo</p>
      )}
    </div>
  )
}
