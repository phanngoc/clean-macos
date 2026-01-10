import { Checkbox } from '@/components/ui/checkbox'
import { formatBytes } from '@/lib/utils'
import type { LargeCacheEntry } from '@/types/cache'
import { HardDrive } from 'lucide-react'

interface LargeCacheItemProps {
  cache: LargeCacheEntry
  isSelected: boolean
  onToggle: () => void
}

export function LargeCacheItem({ cache, isSelected, onToggle }: LargeCacheItemProps) {
  const pathDisplay = cache.path.replace(/^\/Users\/[^/]+/, '~')
  
  return (
    <div
      className={`group flex items-center gap-3 p-4 rounded-lg border transition-all duration-200 cursor-pointer ${
        isSelected 
          ? 'bg-primary/5 border-primary/30 shadow-sm' 
          : 'bg-background border-border hover:border-primary/20 hover:bg-accent/30'
      }`}
      onClick={onToggle}
    >
      <Checkbox checked={isSelected} onCheckedChange={onToggle} className="shrink-0" />
      <HardDrive className="h-4 w-4 text-muted-foreground flex-shrink-0" />
      <div className="flex-1 min-w-0">
        <div className="font-medium text-sm text-foreground truncate mb-0.5">{cache.name}</div>
        <div className="text-xs text-muted-foreground truncate" title={cache.path}>
          {pathDisplay}
        </div>
      </div>
      <div className={`text-sm font-mono font-semibold shrink-0 px-2.5 py-1 rounded-md ${
        isSelected 
          ? 'bg-destructive/10 text-destructive' 
          : 'bg-destructive/5 text-destructive group-hover:bg-destructive/10'
      } transition-colors`}>
        {formatBytes(cache.size_bytes)}
      </div>
    </div>
  )
}
