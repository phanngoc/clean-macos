import { Badge } from '@/components/ui/badge'
import { formatBytes, formatDaysAgo } from '@/lib/utils'
import type { FolderSuggestion } from '@/types/cache'
import { Folder, Clock, Check, AlertTriangle, AlertCircle, Info } from 'lucide-react'

interface SuggestionItemProps {
  suggestion: FolderSuggestion
  isSelected: boolean
  onToggle: () => void
}

function getScoreConfig(score: number) {
  if (score >= 0.7) return {
    variant: 'high' as const,
    label: 'High Risk',
    gradient: 'from-[hsl(8,75%,55%)] to-[hsl(15,75%,50%)]',
    bg: 'bg-[hsl(8,75%,55%)]/10',
    text: 'text-[hsl(8,75%,55%)]',
    icon: AlertTriangle,
  }
  if (score >= 0.4) return {
    variant: 'medium' as const,
    label: 'Medium',
    gradient: 'from-[hsl(50,75%,65%)] to-[hsl(45,70%,60%)]',
    bg: 'bg-[hsl(50,75%,65%)]/10',
    text: 'text-[hsl(45,70%,60%)]',
    icon: AlertCircle,
  }
  return {
    variant: 'low' as const,
    label: 'Low',
    gradient: 'from-[hsl(155,65%,45%)] to-[hsl(165,60%,45%)]',
    bg: 'bg-[hsl(155,65%,45%)]/10',
    text: 'text-[hsl(155,65%,45%)]',
    icon: Info,
  }
}

export function SuggestionItem({ suggestion, isSelected, onToggle }: SuggestionItemProps) {
  const pathDisplay = suggestion.path.replace(/^\/Users\/[^/]+/, '~')
  const scorePercent = Math.round(suggestion.score * 100)
  const scoreConfig = getScoreConfig(suggestion.score)
  const ScoreIcon = scoreConfig.icon
  
  return (
    <div
      className={`group flex items-start gap-4 p-4 rounded-xl border-2 transition-all duration-300 cursor-pointer ${
        isSelected 
          ? `bg-gradient-to-r ${scoreConfig.bg} border-${scoreConfig.text.replace('text-', '')}/40 shadow-lg scale-[1.01]` 
          : 'bg-card/50 border-transparent hover:border-border hover:bg-accent/20 hover:shadow-md'
      }`}
      onClick={onToggle}
    >
      {/* Custom checkbox */}
      <div className={`relative flex items-center justify-center w-8 h-8 rounded-lg transition-all duration-300 mt-0.5 ${
        isSelected 
          ? `bg-gradient-to-br ${scoreConfig.gradient} shadow-md` 
          : 'bg-muted group-hover:bg-muted/80'
      }`}>
        {isSelected && <Check className="h-4 w-4 text-white animate-bounce-in" />}
      </div>

      {/* Icon */}
      <div className={`flex items-center justify-center w-10 h-10 rounded-xl transition-all duration-300 ${scoreConfig.bg}`}>
        <Folder className={`h-5 w-5 transition-colors ${
          isSelected ? scoreConfig.text : 'text-muted-foreground group-hover:text-foreground'
        }`} />
      </div>
      
      <div className="flex-1 min-w-0 space-y-3">
        <div className="flex items-start justify-between gap-3">
          <div className="min-w-0 flex-1">
            <div className={`font-semibold text-sm truncate mb-1 transition-colors ${
              isSelected ? 'text-foreground' : 'text-foreground/80 group-hover:text-foreground'
            }`}>
              {suggestion.name}
            </div>
            <div className="text-xs text-muted-foreground truncate font-mono" title={suggestion.path}>
              {pathDisplay}
            </div>
          </div>
          <div className="flex items-center gap-2 flex-shrink-0">
            {/* Score badge with icon */}
            <div className={`flex items-center gap-1.5 px-2.5 py-1 rounded-lg ${scoreConfig.bg} ${scoreConfig.text} text-xs font-bold`}>
              <ScoreIcon className="h-3.5 w-3.5" />
              {scoreConfig.label} ({scorePercent}%)
            </div>
            {/* Size badge */}
            <div className={`px-3 py-1.5 rounded-lg font-mono text-sm font-bold transition-all duration-300 ${
              isSelected 
                ? `bg-gradient-to-r ${scoreConfig.gradient} text-white shadow-md` 
                : 'bg-muted text-muted-foreground group-hover:bg-muted/80'
            }`}>
              {formatBytes(suggestion.size_bytes)}
            </div>
          </div>
        </div>
        
        {/* Reasons as tags */}
        <div className="flex flex-wrap gap-1.5">
          {suggestion.reasons.map((reason, index) => (
            <Badge 
              key={index} 
              variant="secondary" 
              className="text-xs font-normal px-2 py-0.5 rounded-md bg-muted/50 text-muted-foreground"
            >
              {reason}
            </Badge>
          ))}
        </div>
        
        {/* Last accessed info */}
        {suggestion.last_accessed_days_ago !== undefined && (
          <div className="flex items-center gap-1.5 text-xs text-muted-foreground">
            <Clock className="h-3.5 w-3.5" />
            Last accessed: {formatDaysAgo(suggestion.last_accessed_days_ago)}
          </div>
        )}
      </div>
    </div>
  )
}
