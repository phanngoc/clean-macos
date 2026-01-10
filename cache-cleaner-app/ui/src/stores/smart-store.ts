import { create } from 'zustand'
import { persist } from 'zustand/middleware'
import type { FolderSuggestion } from '@/types/cache'
import { getSmartSuggestions, deleteFolders } from '@/lib/tauri'

interface SmartScannerState {
  // Data
  suggestions: FolderSuggestion[]
  
  // Filters
  minSizeMb: number
  maxAgeDays: number
  
  // Selection
  selectedPaths: Set<string>
  
  // Loading states
  isScanning: boolean
  isDeleting: boolean
  
  // Actions
  scan: () => Promise<void>
  setMinSizeMb: (value: number) => void
  setMaxAgeDays: (value: number) => void
  
  toggleSelection: (path: string) => void
  selectAll: () => void
  deselectAll: () => void
  selectByScoreThreshold: (minScore: number) => void
  
  deleteSelected: () => Promise<{ success: boolean; message: string; bytesTotal: number }>
  
  // Helpers
  getSelectedBytes: () => number
  getSuggestionsByScore: (minScore: number) => FolderSuggestion[]
}

export const useSmartScannerStore = create<SmartScannerState>()(
  persist(
    (set, get) => ({
      // Initial state
      suggestions: [],
      minSizeMb: 100,
      maxAgeDays: 30,
      selectedPaths: new Set(),
      isScanning: false,
      isDeleting: false,
      
      // Actions
      scan: async () => {
        const { minSizeMb, maxAgeDays } = get()
        set({ isScanning: true, selectedPaths: new Set() })
        try {
          const suggestions = await getSmartSuggestions(minSizeMb, maxAgeDays)
          set({ suggestions })
        } finally {
          set({ isScanning: false })
        }
      },
      
      setMinSizeMb: (value) => set({ minSizeMb: value }),
      setMaxAgeDays: (value) => set({ maxAgeDays: value }),
      
      toggleSelection: (path) => {
        const selected = new Set(get().selectedPaths)
        if (selected.has(path)) {
          selected.delete(path)
        } else {
          selected.add(path)
        }
        set({ selectedPaths: selected })
      },
      
      selectAll: () => {
        const selected = new Set(get().suggestions.map(s => s.path))
        set({ selectedPaths: selected })
      },
      
      deselectAll: () => {
        set({ selectedPaths: new Set() })
      },
      
      selectByScoreThreshold: (minScore) => {
        const selected = new Set(
          get().suggestions.filter(s => s.score >= minScore).map(s => s.path)
        )
        set({ selectedPaths: selected })
      },
      
      deleteSelected: async () => {
        const { selectedPaths, suggestions } = get()
        if (selectedPaths.size === 0) {
          return { success: false, message: 'No folders selected', bytesTotal: 0 }
        }
        
        set({ isDeleting: true })
        try {
          const paths = [...selectedPaths]
          const result = await deleteFolders(paths)
          
          // Calculate bytes deleted
          const bytesTotal = suggestions
            .filter(s => selectedPaths.has(s.path))
            .reduce((sum, s) => sum + s.size_bytes, 0)
          
          // Rescan after deletion
          await get().scan()
          set({ selectedPaths: new Set() })
          
          return { success: result.success, message: result.message, bytesTotal }
        } catch (error) {
          return { success: false, message: String(error), bytesTotal: 0 }
        } finally {
          set({ isDeleting: false })
        }
      },
      
      // Helpers
      getSelectedBytes: () => {
        const { suggestions, selectedPaths } = get()
        return suggestions
          .filter(s => selectedPaths.has(s.path))
          .reduce((sum, s) => sum + s.size_bytes, 0)
      },
      
      getSuggestionsByScore: (minScore) => {
        return get().suggestions.filter(s => s.score >= minScore)
      },
    }),
    {
      name: 'smart-scanner-filters',
      partialize: (state) => ({
        minSizeMb: state.minSizeMb,
        maxAgeDays: state.maxAgeDays,
      }),
    }
  )
)
