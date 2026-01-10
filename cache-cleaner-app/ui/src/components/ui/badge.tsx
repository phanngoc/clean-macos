import * as React from "react"
import { cva, type VariantProps } from "class-variance-authority"

import { cn } from "@/lib/utils"

const badgeVariants = cva(
  "inline-flex items-center rounded-lg border px-2.5 py-1 text-xs font-semibold transition-all duration-300 focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2",
  {
    variants: {
      variant: {
        default:
          "border-transparent bg-gradient-to-r from-primary to-[hsl(35,75%,60%)] text-primary-foreground shadow-md",
        secondary:
          "border-transparent bg-secondary text-secondary-foreground hover:bg-secondary/80",
        destructive:
          "border-transparent bg-gradient-to-r from-destructive to-[hsl(15,75%,50%)] text-destructive-foreground shadow-md",
        outline: "text-foreground border-2",
        high: "border-transparent bg-gradient-to-r from-[hsl(8,75%,55%)] to-[hsl(15,75%,50%)] text-white shadow-md",
        medium: "border-transparent bg-gradient-to-r from-[hsl(50,75%,65%)] to-[hsl(45,70%,60%)] text-white shadow-md",
        low: "border-transparent bg-gradient-to-r from-[hsl(155,65%,45%)] to-[hsl(165,60%,45%)] text-white shadow-md",
        info: "border-transparent bg-gradient-to-r from-[hsl(240,70%,55%)] to-[hsl(280,65%,45%)] text-white shadow-md",
      },
    },
    defaultVariants: {
      variant: "default",
    },
  }
)

export interface BadgeProps
  extends React.HTMLAttributes<HTMLDivElement>,
    VariantProps<typeof badgeVariants> {}

function Badge({ className, variant, ...props }: BadgeProps) {
  return (
    <div className={cn(badgeVariants({ variant }), className)} {...props} />
  )
}

export { Badge, badgeVariants }
