export interface GelbooruTag {
  id: number
  name: string
  count: number
  type: number // 0=general, 1=artist, 3=copyright, 4=character, 5=metadata
  ambiguous: boolean
}

export interface GelbooruTagSearchResponse {
  tags: GelbooruTag[]
  query: string
}

/** Tag type to display color mapping */
export const TAG_TYPE_COLORS: Record<number, { bg: string; text: string; label: string }> = {
  0: { bg: 'bg-blue-500/20', text: 'text-blue-400', label: 'General' },
  1: { bg: 'bg-red-500/20', text: 'text-red-400', label: 'Artist' },
  3: { bg: 'bg-purple-500/20', text: 'text-purple-400', label: 'Copyright' },
  4: { bg: 'bg-green-500/20', text: 'text-green-400', label: 'Character' },
  5: { bg: 'bg-orange-500/20', text: 'text-orange-400', label: 'Metadata' },
}
