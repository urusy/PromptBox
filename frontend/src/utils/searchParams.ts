import type { ImageSearchParams } from '@/types/image'

export const DEFAULT_SEARCH_PARAMS: ImageSearchParams = {
  page: 1,
  per_page: 24,
  sort_by: 'created_at',
  sort_order: 'desc',
}

// Parse URL search params to ImageSearchParams
export function parseSearchParams(searchParams: URLSearchParams): ImageSearchParams {
  const params: ImageSearchParams = { ...DEFAULT_SEARCH_PARAMS }

  const q = searchParams.get('q')
  if (q) params.q = q

  const source_tool = searchParams.get('source_tool')
  if (source_tool) params.source_tool = source_tool

  const model_type = searchParams.get('model_type')
  if (model_type) params.model_type = model_type

  const model_name = searchParams.get('model_name')
  if (model_name) params.model_name = model_name

  const sampler_name = searchParams.get('sampler_name')
  if (sampler_name) params.sampler_name = sampler_name

  const min_rating = searchParams.get('min_rating')
  if (min_rating) params.min_rating = parseInt(min_rating)

  const exact_rating = searchParams.get('exact_rating')
  if (exact_rating) params.exact_rating = parseInt(exact_rating)

  const is_favorite = searchParams.get('is_favorite')
  if (is_favorite === 'true') params.is_favorite = true

  const needs_improvement = searchParams.get('needs_improvement')
  if (needs_improvement === 'true') params.needs_improvement = true

  const tags = searchParams.get('tags')
  if (tags) params.tags = tags.split(',')

  const lora_name = searchParams.get('lora_name')
  if (lora_name) params.lora_name = lora_name

  const is_xyz_grid = searchParams.get('is_xyz_grid')
  if (is_xyz_grid === 'true') params.is_xyz_grid = true
  else if (is_xyz_grid === 'false') params.is_xyz_grid = false

  const is_upscaled = searchParams.get('is_upscaled')
  if (is_upscaled === 'true') params.is_upscaled = true
  else if (is_upscaled === 'false') params.is_upscaled = false

  const min_width = searchParams.get('min_width')
  if (min_width) params.min_width = parseInt(min_width)

  const min_height = searchParams.get('min_height')
  if (min_height) params.min_height = parseInt(min_height)

  const page = searchParams.get('page')
  if (page) params.page = parseInt(page)

  const per_page = searchParams.get('per_page')
  if (per_page) params.per_page = parseInt(per_page)

  const sort_by = searchParams.get('sort_by')
  if (sort_by) params.sort_by = sort_by

  const sort_order = searchParams.get('sort_order')
  if (sort_order === 'asc' || sort_order === 'desc') params.sort_order = sort_order

  return params
}

// Convert ImageSearchParams to URLSearchParams
export function toSearchParams(params: ImageSearchParams): URLSearchParams {
  const searchParams = new URLSearchParams()

  if (params.q) searchParams.set('q', params.q)
  if (params.source_tool) searchParams.set('source_tool', params.source_tool)
  if (params.model_type) searchParams.set('model_type', params.model_type)
  if (params.model_name) searchParams.set('model_name', params.model_name)
  if (params.sampler_name) searchParams.set('sampler_name', params.sampler_name)
  if (params.min_rating !== undefined) searchParams.set('min_rating', params.min_rating.toString())
  if (params.exact_rating !== undefined) searchParams.set('exact_rating', params.exact_rating.toString())
  if (params.is_favorite === true) searchParams.set('is_favorite', 'true')
  if (params.needs_improvement === true) searchParams.set('needs_improvement', 'true')
  if (params.tags && params.tags.length > 0) searchParams.set('tags', params.tags.join(','))
  if (params.lora_name) searchParams.set('lora_name', params.lora_name)
  if (params.is_xyz_grid === true) searchParams.set('is_xyz_grid', 'true')
  else if (params.is_xyz_grid === false) searchParams.set('is_xyz_grid', 'false')
  if (params.is_upscaled === true) searchParams.set('is_upscaled', 'true')
  else if (params.is_upscaled === false) searchParams.set('is_upscaled', 'false')
  if (params.min_width) searchParams.set('min_width', params.min_width.toString())
  if (params.min_height) searchParams.set('min_height', params.min_height.toString())
  if (params.page && params.page !== 1) searchParams.set('page', params.page.toString())
  if (params.per_page && params.per_page !== 24) searchParams.set('per_page', params.per_page.toString())
  if (params.sort_by && params.sort_by !== 'created_at') searchParams.set('sort_by', params.sort_by)
  if (params.sort_order && params.sort_order !== 'desc') searchParams.set('sort_order', params.sort_order)

  return searchParams
}

// Convert ImageSearchParams to API query params (without pagination for detail page)
export function toApiParams(params: ImageSearchParams): Record<string, string | number | boolean | undefined> {
  const apiParams: Record<string, string | number | boolean | undefined> = {}

  if (params.q) apiParams.q = params.q
  if (params.source_tool) apiParams.source_tool = params.source_tool
  if (params.model_type) apiParams.model_type = params.model_type
  if (params.model_name) apiParams.model_name = params.model_name
  if (params.sampler_name) apiParams.sampler_name = params.sampler_name
  if (params.min_rating !== undefined) apiParams.min_rating = params.min_rating
  if (params.exact_rating !== undefined) apiParams.exact_rating = params.exact_rating
  if (params.is_favorite !== undefined) apiParams.is_favorite = params.is_favorite
  if (params.needs_improvement !== undefined) apiParams.needs_improvement = params.needs_improvement
  if (params.lora_name) apiParams.lora_name = params.lora_name
  if (params.is_xyz_grid !== undefined && params.is_xyz_grid !== null) apiParams.is_xyz_grid = params.is_xyz_grid
  if (params.is_upscaled !== undefined && params.is_upscaled !== null) apiParams.is_upscaled = params.is_upscaled
  if (params.min_width !== undefined) apiParams.min_width = params.min_width
  if (params.min_height !== undefined) apiParams.min_height = params.min_height
  if (params.include_deleted) apiParams.include_deleted = params.include_deleted
  if (params.sort_by && params.sort_by !== 'created_at') apiParams.sort_by = params.sort_by
  if (params.sort_order && params.sort_order !== 'desc') apiParams.sort_order = params.sort_order

  return apiParams
}
