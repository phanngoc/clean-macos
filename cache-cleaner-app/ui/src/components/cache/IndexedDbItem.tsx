import { formatBytes } from '@/lib/utils'
import type { IndexedDbItem } from '@/types/cache'
import { Database, Check, Globe } from 'lucide-react'

interface IndexedDbItemProps {
  item: IndexedDbItem
  isSelected: boolean
  onToggle: () => void
}

export function IndexedDbItemComponent({ item, isSelected, onToggle }: IndexedDbItemProps) {
  // Extract domain name for display
  const getDomainDisplay = (origin: string) => {
    try {
      if (origin.startsWith('http')) {
        return new URL(origin).hostname
      }
      return origin.split('_')[0] || origin
    } catch {
      return origin
    }
  }
  
  const domainDisplay = getDomainDisplay(item.origin)
  const isExtension = item.origin.includes('chrome-extension')
  
  return (
    <div
      className={`group flex items-center gap-4 p-4 rounded-xl border-2 transition-all duration-300 cursor-pointer ${
        isSelected 
          ? 'bg-gradient-to-r from-[hsl(280,70%,55%)]/5 to-[hsl(280,70%,55%)]/10 border-[hsl(280,70%,55%)]/40 shadow-lg shadow-[hsl(280,70%,55%)]/10 scale-[1.01]' 
          : 'bg-card/50 border-transparent hover:border-border hover:bg-accent/20 hover:shadow-md'
      }`}
      onClick={onToggle}
    >
      {/* Custom checkbox area */}
      <div className={`relative flex items-center justify-center w-8 h-8 rounded-lg transition-all duration-300 ${
        isSelected 
          ? 'bg-gradient-to-br from-[hsl(280,70%,55%)] to-[hsl(310,65%,45%)] shadow-md' 
          : 'bg-muted group-hover:bg-muted/80'
      }`}>
        {isSelected && <Check className="h-4 w-4 text-white animate-bounce-in" />}
      </div>

      {/* Icon */}
      <div className="flex items-center justify-center w-10 h-10 rounded-xl bg-gradient-to-br from-[hsl(280,70%,55%)]/10 to-[hsl(280,70%,55%)]/20">
        {isExtension ? (
          <Database className={`h-5 w-5 transition-colors ${
            isSelected ? 'text-[hsl(280,70%,55%)]' : 'text-muted-foreground group-hover:text-foreground'
          }`} />
        ) : (
          <Globe className={`h-5 w-5 transition-colors ${
            isSelected ? 'text-[hsl(280,70%,55%)]' : 'text-muted-foreground group-hover:text-foreground'
          }`} />
        )}
      </div>
      
      {/* Content */}
      <div className="flex-1 min-w-0">
        <div className={`font-semibold text-sm truncate mb-1 transition-colors ${
          isSelected ? 'text-foreground' : 'text-foreground/80 group-hover:text-foreground'
        }`}>
          {domainDisplay}
        </div>
        <div className="flex items-center gap-2 text-xs text-muted-foreground">
          <span className="px-2 py-0.5 rounded-md bg-muted/50 font-medium">
            {item.profile}
          </span>
          {isExtension && (
            <span className="px-2 py-0.5 rounded-md bg-[hsl(240,70%,55%)]/10 text-[hsl(240,70%,55%)] font-medium">
              Extension
            </span>
          )}
        </div>
      </div>
      
      {/* Size badge */}
      <div className={`shrink-0 px-3 py-1.5 rounded-lg font-mono text-sm font-bold transition-all duration-300 ${
        isSelected 
          ? 'bg-gradient-to-r from-[hsl(280,70%,55%)] to-[hsl(310,65%,45%)] text-white shadow-md' 
          : item.over_threshold
            ? 'bg-[hsl(280,70%,55%)]/10 text-[hsl(280,70%,55%)] group-hover:bg-[hsl(280,70%,55%)]/15'
            : 'bg-muted text-muted-foreground group-hover:bg-muted/80'
      }`}>
        {formatBytes(item.size)}
      </div>
    </div>
  )
}
