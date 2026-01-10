import { Checkbox } from '@/components/ui/checkbox'
import { Badge } from '@/components/ui/badge'
import { formatBytes, formatDaysAgo } from '@/lib/utils'
import type { FolderSuggestion } from '@/types/cache'
import { Folder, Clock } from 'lucide-react'

interface SuggestionItemProps {
  suggestion: FolderSuggestion
  isSelected: boolean
  onToggle: () => void
}

function getScoreVariant(score: number): 'high' | 'medium' | 'low' {
  if (score >= 0.7) return 'high'
  if (score >= 0.4) return 'medium'
  return 'low'
}

function getScoreLabel(score: number): string {
  if (score >= 0.7) return 'High'
  if (score >= 0.4) return 'Medium'
  return 'Low'
}

export function SuggestionItem({ suggestion, isSelected, onToggle }: SuggestionItemProps) {
  const pathDisplay = suggestion.path.replace(/^\/Users\/[^/]+/, '~')
  const scorePercent = Math.round(suggestion.score * 100)
  
  return (
    <div
      className={`group flex items-start gap-3 p-4 rounded-lg border transition-all duration-200 cursor-pointer ${
        isSelected 
          ? 'bg-primary/5 border-primary/30 shadow-sm' 
          : 'bg-background border-border hover:border-primary/20 hover:bg-accent/30'
      }`}
      onClick={onToggle}
    >
      <Checkbox checked={isSelected} onCheckedChange={onToggle} className="mt-1 shrink-0" />
      <Folder className="h-5 w-5 text-muted-foreground flex-shrink-0 mt-0.5" />
      <div className="flex-1 min-w-0 space-y-2.5">
        <div className="flex items-start justify-between gap-3">
          <div className="min-w-0 flex-1">
            <div className="font-medium text-sm text-foreground truncate mb-0.5">{suggestion.name}</div>
            <div className="text-xs text-muted-foreground truncate" title={suggestion.path}>
              {pathDisplay}
            </div>
          </div>
          <div className="flex items-center gap-2 flex-shrink-0">
            <Badge variant={getScoreVariant(suggestion.score)} className="text-xs">
              {getScoreLabel(suggestion.score)} ({scorePercent}%)
            </Badge>
            <div className={`text-sm font-mono font-semibold px-2.5 py-1 rounded-md ${
              isSelected 
                ? 'bg-primary/10 text-primary' 
                : 'bg-muted text-muted-foreground group-hover:bg-muted/80'
            } transition-colors`}>
              {formatBytes(suggestion.size_bytes)}
            </div>
          </div>
        </div>
        
        <div className="flex flex-wrap gap-1.5">
          {suggestion.reasons.map((reason, index) => (
            <Badge key={index} variant="secondary" className="text-xs font-normal">
              {reason}
            </Badge>
          ))}
        </div>
        
        {suggestion.last_accessed_days_ago !== undefined && (
          <div className="flex items-center gap-1.5 text-xs text-muted-foreground">
            <Clock className="h-3 w-3" />
            Last accessed: {formatDaysAgo(suggestion.last_accessed_days_ago)}
          </div>
        )}
      </div>
    </div>
  )
}
