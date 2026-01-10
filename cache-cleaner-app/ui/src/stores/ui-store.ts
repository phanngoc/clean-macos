import { create } from 'zustand'

export type TabId = 'cache-cleaner' | 'smart-scanner'

interface StatusMessage {
  type: 'success' | 'error' | 'info'
  text: string
  timestamp: number
}

interface UiState {
  // Tab state
  activeTab: TabId
  setActiveTab: (tab: TabId) => void
  
  // Status messages
  statusMessage: StatusMessage | null
  setStatus: (type: StatusMessage['type'], text: string) => void
  clearStatus: () => void
  
  // Global loading state
  isLoading: boolean
  setLoading: (loading: boolean) => void
}

export const useUiStore = create<UiState>((set) => ({
  // Tab state
  activeTab: 'cache-cleaner',
  setActiveTab: (tab) => set({ activeTab: tab }),
  
  // Status messages
  statusMessage: null,
  setStatus: (type, text) => set({
    statusMessage: { type, text, timestamp: Date.now() }
  }),
  clearStatus: () => set({ statusMessage: null }),
  
  // Global loading
  isLoading: false,
  setLoading: (loading) => set({ isLoading: loading }),
}))
