import { useUiStore } from '@/stores'
import { cn } from '@/lib/utils'
import { CheckCircle, XCircle, Info, X, Sparkles } from 'lucide-react'
import { useEffect } from 'react'

export function StatusMessage() {
  const { statusMessage, clearStatus } = useUiStore()

  // Auto-dismiss after 5 seconds
  useEffect(() => {
    if (statusMessage) {
      const timer = setTimeout(clearStatus, 5000)
      return () => clearTimeout(timer)
    }
  }, [statusMessage, clearStatus])

  if (!statusMessage) return null

  const configs = {
    success: {
      icon: <CheckCircle className="h-5 w-5" />,
      gradient: 'from-[oklch(0.65_0.2_145)] to-[oklch(0.6_0.18_165)]',
      bg: 'from-[oklch(0.65_0.2_145)]/10 to-[oklch(0.6_0.18_165)]/10',
      border: 'border-[oklch(0.65_0.2_145)]/30',
      iconColor: 'text-[oklch(0.65_0.2_145)]',
      shadow: 'shadow-[oklch(0.65_0.2_145)]/20',
    },
    error: {
      icon: <XCircle className="h-5 w-5" />,
      gradient: 'from-[oklch(0.6_0.22_15)] to-[oklch(0.55_0.2_25)]',
      bg: 'from-[oklch(0.6_0.22_15)]/10 to-[oklch(0.55_0.2_25)]/10',
      border: 'border-[oklch(0.6_0.22_15)]/30',
      iconColor: 'text-[oklch(0.6_0.22_15)]',
      shadow: 'shadow-[oklch(0.6_0.22_15)]/20',
    },
    info: {
      icon: <Info className="h-5 w-5" />,
      gradient: 'from-[oklch(0.6_0.2_260)] to-[oklch(0.55_0.18_280)]',
      bg: 'from-[oklch(0.6_0.2_260)]/10 to-[oklch(0.55_0.18_280)]/10',
      border: 'border-[oklch(0.6_0.2_260)]/30',
      iconColor: 'text-[oklch(0.6_0.2_260)]',
      shadow: 'shadow-[oklch(0.6_0.2_260)]/20',
    },
  }

  const config = configs[statusMessage.type]

  return (
    <div
      className={cn(
        'fixed bottom-6 right-6 flex items-center gap-4 px-5 py-4 rounded-2xl border-2 backdrop-blur-xl z-50 max-w-md animate-bounce-in',
        `bg-gradient-to-r ${config.bg}`,
        config.border,
        `shadow-xl ${config.shadow}`
      )}
    >
      {/* Icon with gradient background */}
      <div className={cn(
        'flex items-center justify-center w-10 h-10 rounded-xl bg-gradient-to-br shadow-lg',
        config.gradient
      )}>
        <div className="text-white">
          {statusMessage.type === 'success' ? <Sparkles className="h-5 w-5" /> : config.icon}
        </div>
      </div>
      
      {/* Message */}
      <div className="flex-1">
        <p className="font-semibold text-foreground">
          {statusMessage.type === 'success' ? 'Success!' : statusMessage.type === 'error' ? 'Error' : 'Info'}
        </p>
        <p className="text-sm text-muted-foreground">{statusMessage.text}</p>
      </div>
      
      {/* Close button */}
      <button
        onClick={clearStatus}
        className="p-2 rounded-xl hover:bg-foreground/10 transition-all duration-300 cursor-pointer hover:scale-110 active:scale-95"
        aria-label="Close notification"
      >
        <X className="h-4 w-4 text-muted-foreground" />
      </button>
      
      {/* Progress bar for auto-dismiss */}
      <div className="absolute bottom-0 left-4 right-4 h-1 rounded-full overflow-hidden bg-foreground/5">
        <div 
          className={cn('h-full rounded-full bg-gradient-to-r', config.gradient)}
          style={{
            animation: 'shrink 5s linear forwards',
          }}
        />
      </div>
      
      <style>{`
        @keyframes shrink {
          from { width: 100%; }
          to { width: 0%; }
        }
      `}</style>
    </div>
  )
}
