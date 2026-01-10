import { formatBytes } from '@/lib/utils'
import type { LargeCacheEntry } from '@/types/cache'
import { HardDrive, Check, AlertTriangle } from 'lucide-react'

interface LargeCacheItemProps {
  cache: LargeCacheEntry
  isSelected: boolean
  onToggle: () => void
}

export function LargeCacheItem({ cache, isSelected, onToggle }: LargeCacheItemProps) {
  const pathDisplay = cache.path.replace(/^\/Users\/[^/]+/, '~')
  const sizeGB = cache.size_bytes / (1024 * 1024 * 1024)
  
  return (
    <div
      className={`group flex items-center gap-4 p-4 rounded-xl border-2 transition-all duration-300 cursor-pointer ${
        isSelected 
          ? 'bg-gradient-to-r from-[oklch(0.6_0.22_15)]/5 to-[oklch(0.6_0.22_15)]/10 border-[oklch(0.6_0.22_15)]/40 shadow-lg shadow-[oklch(0.6_0.22_15)]/10 scale-[1.01]' 
          : 'bg-card/50 border-transparent hover:border-border hover:bg-accent/20 hover:shadow-md'
      }`}
      onClick={onToggle}
    >
      {/* Custom checkbox area */}
      <div className={`relative flex items-center justify-center w-8 h-8 rounded-lg transition-all duration-300 ${
        isSelected 
          ? 'bg-gradient-to-br from-[oklch(0.6_0.22_15)] to-[oklch(0.55_0.2_25)] shadow-md' 
          : 'bg-muted group-hover:bg-muted/80'
      }`}>
        {isSelected && <Check className="h-4 w-4 text-white animate-bounce-in" />}
      </div>

      {/* Icon with warning indicator */}
      <div className="relative flex items-center justify-center w-10 h-10 rounded-xl bg-gradient-to-br from-[oklch(0.6_0.22_15)]/10 to-[oklch(0.6_0.22_15)]/20">
        <HardDrive className={`h-5 w-5 transition-colors ${
          isSelected ? 'text-[oklch(0.6_0.22_15)]' : 'text-muted-foreground group-hover:text-foreground'
        }`} />
        {sizeGB > 2 && (
          <div className="absolute -top-1 -right-1 w-4 h-4 rounded-full bg-[oklch(0.75_0.18_70)] flex items-center justify-center">
            <AlertTriangle className="h-2.5 w-2.5 text-white" />
          </div>
        )}
      </div>
      
      {/* Content */}
      <div className="flex-1 min-w-0">
        <div className={`font-semibold text-sm truncate mb-1 transition-colors ${
          isSelected ? 'text-foreground' : 'text-foreground/80 group-hover:text-foreground'
        }`}>
          {cache.name}
        </div>
        <div className="text-xs text-muted-foreground truncate font-mono" title={cache.path}>
          {pathDisplay}
        </div>
      </div>
      
      {/* Size badge - prominent for large caches */}
      <div className={`shrink-0 px-3 py-1.5 rounded-lg font-mono text-sm font-bold transition-all duration-300 ${
        isSelected 
          ? 'bg-gradient-to-r from-[oklch(0.6_0.22_15)] to-[oklch(0.55_0.2_25)] text-white shadow-md' 
          : 'bg-[oklch(0.6_0.22_15)]/10 text-[oklch(0.6_0.22_15)] group-hover:bg-[oklch(0.6_0.22_15)]/15'
      }`}>
        {formatBytes(cache.size_bytes)}
      </div>
    </div>
  )
}
