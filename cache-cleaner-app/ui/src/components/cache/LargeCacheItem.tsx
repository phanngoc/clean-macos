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
          ? 'bg-gradient-to-r from-[hsl(8,75%,55%)]/5 to-[hsl(8,75%,55%)]/10 border-[hsl(8,75%,55%)]/40 shadow-lg shadow-[hsl(8,75%,55%)]/10 scale-[1.01]' 
          : 'bg-card/50 border-transparent hover:border-border hover:bg-accent/20 hover:shadow-md'
      }`}
      onClick={onToggle}
    >
      {/* Custom checkbox area */}
      <div className={`relative flex items-center justify-center w-8 h-8 rounded-lg transition-all duration-300 ${
        isSelected 
          ? 'bg-gradient-to-br from-[hsl(8,75%,55%)] to-[hsl(15,75%,50%)] shadow-md' 
          : 'bg-muted group-hover:bg-muted/80'
      }`}>
        {isSelected && <Check className="h-4 w-4 text-white animate-bounce-in" />}
      </div>

      {/* Icon with warning indicator */}
      <div className="relative flex items-center justify-center w-10 h-10 rounded-xl bg-gradient-to-br from-[hsl(8,75%,55%)]/10 to-[hsl(8,75%,55%)]/20">
        <HardDrive className={`h-5 w-5 transition-colors ${
          isSelected ? 'text-[hsl(8,75%,55%)]' : 'text-muted-foreground group-hover:text-foreground'
        }`} />
        {sizeGB > 2 && (
          <div className="absolute -top-1 -right-1 w-4 h-4 rounded-full bg-[hsl(50,75%,65%)] flex items-center justify-center">
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
          ? 'bg-gradient-to-r from-[hsl(8,75%,55%)] to-[hsl(15,75%,50%)] text-white shadow-md' 
          : 'bg-[hsl(8,75%,55%)]/10 text-[hsl(8,75%,55%)] group-hover:bg-[hsl(8,75%,55%)]/15'
      }`}>
        {formatBytes(cache.size_bytes)}
      </div>
    </div>
  )
}
