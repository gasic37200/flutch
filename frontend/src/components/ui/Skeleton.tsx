/** 로딩 자리표시자. 크기는 className으로(예: `h-4 w-32`, `aspect-video w-full`). */
export default function Skeleton({ className = '' }: { className?: string }) {
  return <div aria-hidden className={`animate-pulse rounded-xs bg-surface-2 motion-reduce:animate-none ${className}`} />
}
