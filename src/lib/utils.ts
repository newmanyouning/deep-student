// Re-export from the canonical cn() implementation with twMerge support.
// This file exists for backward compatibility; all ~250 legacy consumers
// now automatically get Tailwind class conflict resolution.
export { cn, type ClassValue } from "@/utils/cn"
