import { Moon, Sun, Monitor } from 'lucide-react'
import { useTheme } from '@/components/theme-provider'

export function ThemeToggle() {
  const { theme, setTheme } = useTheme()

  const cycleTheme = () => {
    if (theme === 'light') {
      setTheme('dark')
    } else if (theme === 'dark') {
      setTheme('system')
    } else {
      setTheme('light')
    }
  }

  const getIcon = () => {
    if (theme === 'light') return <Sun className="h-5 w-5" />
    if (theme === 'dark') return <Moon className="h-5 w-5" />
    return <Monitor className="h-5 w-5" />
  }

  const getLabel = () => {
    if (theme === 'light') return 'Light'
    if (theme === 'dark') return 'Dark'
    return 'System'
  }

  return (
    <button
      onClick={cycleTheme}
      title={`Current: ${theme} (click to change)`}
      className="flex items-center gap-2 px-4 py-2.5 rounded-xl bg-muted/50 hover:bg-muted border-2 border-transparent hover:border-border transition-all duration-300 cursor-pointer hover:scale-[1.02] active:scale-[0.98]"
    >
      <div className="flex items-center justify-center w-8 h-8 rounded-lg bg-gradient-to-br from-[hsl(50,75%,65%)] to-[hsl(35,70%,60%)] text-white shadow-md">
        {getIcon()}
      </div>
      <span className="text-sm font-medium text-muted-foreground">{getLabel()}</span>
      <span className="sr-only">Toggle theme</span>
    </button>
  )
}
