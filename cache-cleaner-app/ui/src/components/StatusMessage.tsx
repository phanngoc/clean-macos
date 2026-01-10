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
      gradient: 'from-[hsl(155,70%,55%)] to-[hsl(165,65%,50%)]',
      bg: 'from-[hsl(155,70%,55%)]/10 to-[hsl(165,65%,50%)]/10',
      border: 'border-[hsl(155,70%,55%)]/30',
      iconColor: 'text-[hsl(155,70%,55%)]',
      shadow: 'shadow-[hsl(155,70%,55%)]/20',
    },
    error: {
      icon: <XCircle className="h-5 w-5" />,
      gradient: 'from-[hsl(8,75%,55%)] to-[hsl(15,75%,50%)]',
      bg: 'from-[hsl(8,75%,55%)]/10 to-[hsl(15,75%,50%)]/10',
      border: 'border-[hsl(8,75%,55%)]/30',
      iconColor: 'text-[hsl(8,75%,55%)]',
      shadow: 'shadow-[hsl(8,75%,55%)]/20',
    },
    info: {
      icon: <Info className="h-5 w-5" />,
      gradient: 'from-[hsl(240,70%,55%)] to-[hsl(280,65%,45%)]',
      bg: 'from-[hsl(240,70%,55%)]/10 to-[hsl(280,65%,45%)]/10',
      border: 'border-[hsl(240,70%,55%)]/30',
      iconColor: 'text-[hsl(240,70%,55%)]',
      shadow: 'shadow-[hsl(240,70%,55%)]/20',
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
