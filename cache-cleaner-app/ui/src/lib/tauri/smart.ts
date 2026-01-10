import { invoke } from '@tauri-apps/api/core'
import type { FolderSuggestion, CleanResult } from '@/types/cache'

/**
 * Get AI-scored smart suggestions for folders to clean
 * @param minSizeMb - Minimum size in MB (default: 100)
 * @param maxAgeDays - Maximum age in days to consider (default: 30)
 */
export async function getSmartSuggestions(
  minSizeMb?: number,
  maxAgeDays?: number
): Promise<FolderSuggestion[]> {
  return invoke<FolderSuggestion[]>('get_smart_suggestions', { minSizeMb, maxAgeDays })
}

/**
 * Delete folders by their paths
 * @param paths - Array of folder paths to delete
 */
export async function deleteFolders(paths: string[]): Promise<CleanResult> {
  return invoke<CleanResult>('delete_folders', { paths })
}
