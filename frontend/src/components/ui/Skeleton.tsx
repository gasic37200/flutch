/** 로딩 자리표시자. 크기는 className으로(예: h-4 w-32, aspect-video w-full). card = 이미지 자리(12px 사선). */
export default function Skeleton({ className = '', card }: { className?: string; card?: boolean }) {
  return <div aria-hidden className={`animate-shimmer bg-surface-2 motion-reduce:animate-none ${card ? 'chamfer [--c:12px]' : ''} ${className}`} />
}
