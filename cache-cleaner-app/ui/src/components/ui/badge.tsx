import * as React from "react"
import { cva, type VariantProps } from "class-variance-authority"

import { cn } from "@/lib/utils"

const badgeVariants = cva(
  "inline-flex items-center rounded-lg border px-2.5 py-1 text-xs font-semibold transition-all duration-300 focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2",
  {
    variants: {
      variant: {
        default:
          "border-transparent bg-gradient-to-r from-primary to-[oklch(0.7_0.18_50)] text-primary-foreground shadow-md",
        secondary:
          "border-transparent bg-secondary text-secondary-foreground hover:bg-secondary/80",
        destructive:
          "border-transparent bg-gradient-to-r from-destructive to-[oklch(0.55_0.2_25)] text-destructive-foreground shadow-md",
        outline: "text-foreground border-2",
        high: "border-transparent bg-gradient-to-r from-[oklch(0.6_0.22_15)] to-[oklch(0.55_0.2_25)] text-white shadow-md",
        medium: "border-transparent bg-gradient-to-r from-[oklch(0.75_0.18_70)] to-[oklch(0.7_0.16_55)] text-white shadow-md",
        low: "border-transparent bg-gradient-to-r from-[oklch(0.6_0.18_145)] to-[oklch(0.55_0.16_165)] text-white shadow-md",
        info: "border-transparent bg-gradient-to-r from-[oklch(0.6_0.2_260)] to-[oklch(0.55_0.18_280)] text-white shadow-md",
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
