import { Checkbox } from '@/components/ui/checkbox'
import { formatBytes } from '@/lib/utils'
import type { CacheInfo } from '@/types/cache'
import { Folder, Check } from 'lucide-react'

interface CacheItemProps {
  cache: CacheInfo
  isSelected: boolean
  onToggle: () => void
}

export function CacheItem({ cache, isSelected, onToggle }: CacheItemProps) {
  const pathDisplay = cache.path.replace(/^\/Users\/[^/]+/, '~')
  const displayName = cache.cache_type.replace(/_/g, ' ').replace(/\b\w/g, c => c.toUpperCase())
  
  // Size indicator color based on size
  const getSizeColor = (size: number) => {
    if (size > 1024 * 1024 * 500) return 'from-[hsl(8,75%,55%)] to-[hsl(15,75%,55%)]' // > 500MB - coral
    if (size > 1024 * 1024 * 100) return 'from-[hsl(35,75%,60%)] to-[hsl(50,70%,65%)]' // > 100MB - orange
    if (size > 1024 * 1024 * 10) return 'from-[hsl(70,70%,55%)] to-[hsl(85,70%,60%)]' // > 10MB - yellow
    return 'from-[hsl(155,65%,45%)] to-[hsl(165,65%,50%)]' // green
  }
  
  return (
    <div
      className={`group flex items-center gap-4 p-4 rounded-xl border-2 transition-all duration-300 cursor-pointer ${
        isSelected 
          ? 'bg-gradient-to-r from-primary/5 to-primary/10 border-primary/40 shadow-lg shadow-primary/10 scale-[1.01]' 
          : 'bg-card/50 border-transparent hover:border-border hover:bg-accent/20 hover:shadow-md'
      } ${!cache.exists ? 'opacity-40 pointer-events-none' : ''}`}
      onClick={onToggle}
    >
      {/* Custom checkbox area */}
      <div className={`relative flex items-center justify-center w-8 h-8 rounded-lg transition-all duration-300 ${
        isSelected 
          ? 'bg-gradient-to-br from-primary to-[hsl(35,75%,60%)] shadow-md' 
          : 'bg-muted group-hover:bg-muted/80'
      }`}>
        {isSelected ? (
          <Check className="h-4 w-4 text-white animate-bounce-in" />
        ) : (
          <Checkbox
            checked={false}
            onCheckedChange={onToggle}
            disabled={!cache.exists}
            className="opacity-0"
          />
        )}
      </div>

      {/* Icon */}
      <div className={`flex items-center justify-center w-10 h-10 rounded-xl transition-all duration-300 ${
        isSelected 
          ? 'bg-primary/10' 
          : 'bg-muted/50 group-hover:bg-muted'
      }`}>
        <Folder className={`h-5 w-5 transition-colors ${
          isSelected ? 'text-primary' : 'text-muted-foreground group-hover:text-foreground'
        }`} />
      </div>
      
      {/* Content */}
      <div className="flex-1 min-w-0">
        <div className={`font-semibold text-sm truncate mb-1 transition-colors ${
          isSelected ? 'text-foreground' : 'text-foreground/80 group-hover:text-foreground'
        }`}>
          {displayName}
        </div>
        <div className="text-xs text-muted-foreground truncate font-mono" title={cache.path}>
          {pathDisplay}
        </div>
      </div>
      
      {/* Size badge with gradient */}
      <div className={`shrink-0 px-3 py-1.5 rounded-lg font-mono text-sm font-bold transition-all duration-300 ${
        isSelected 
          ? `bg-gradient-to-r ${getSizeColor(cache.size)} text-white shadow-md` 
          : 'bg-muted text-muted-foreground group-hover:bg-muted/80'
      }`}>
        {cache.exists ? formatBytes(cache.size) : 'Not found'}
      </div>
    </div>
  )
}
