import { useUiStore } from '@/stores'
import { cn } from '@/lib/utils'
import { CheckCircle, XCircle, Info, X } from 'lucide-react'
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

  const icons = {
    success: <CheckCircle className="h-5 w-5 text-green-500" />,
    error: <XCircle className="h-5 w-5 text-red-500" />,
    info: <Info className="h-5 w-5 text-blue-500" />,
  }

  const backgrounds = {
    success: 'bg-green-500/10 border-green-500/20',
    error: 'bg-red-500/10 border-red-500/20',
    info: 'bg-blue-500/10 border-blue-500/20',
  }

  return (
    <div
      className={cn(
        'fixed bottom-6 right-6 flex items-center gap-3 px-4 py-3 rounded-xl border shadow-lg backdrop-blur-sm animate-in slide-in-from-bottom-5 duration-300 z-50 max-w-md',
        backgrounds[statusMessage.type]
      )}
    >
      {icons[statusMessage.type]}
      <p className="text-sm font-medium flex-1">{statusMessage.text}</p>
      <button
        onClick={clearStatus}
        className="ml-2 p-1 rounded-lg hover:bg-foreground/10 transition-colors cursor-pointer"
        aria-label="Close notification"
      >
        <X className="h-4 w-4" />
      </button>
    </div>
  )
}
