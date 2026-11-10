import type { HTMLAttributes } from 'react'

type Props = HTMLAttributes<HTMLDivElement> & {
  /** line=기본 은은한 네온, hero=메인 히어로·후원 알림(테두리+글로우), clutch=SUPER CLUTCH 그라데이션 */
  tone?: 'line' | 'hero' | 'clutch'
  /** lg=12px 큰 카드·패널, sm=6px */
  size?: 'lg' | 'sm'
  /** 호버 시 밝은 네온 테두리 */
  interactive?: boolean
  selected?: boolean
  /** bg-*를 주면 기본 배경(surface)을 대신한다 */
  innerClassName?: string
}

// 사선 모서리 패널. 바깥 층(테두리색)과 1px 안쪽 층(배경)을 clip-path로 깎고, 글로우·포커스는 clip-path 밖 래퍼에 건다.
export default function Panel({ tone = 'line', size = 'lg', interactive, selected, className = '', innerClassName = '', children, ...rest }: Props) {
  const edge =
    tone === 'clutch' ? 'bg-clutch' : tone === 'hero' || selected ? 'bg-brand' : interactive ? 'bg-brand-line hover:bg-brand' : 'bg-brand-line'
  const glow = tone === 'hero' ? 'drop-shadow-glow motion-reduce:drop-shadow-none' : 'has-[:focus-visible]:drop-shadow-focus'
  // 같은 속성의 클래스끼리는 CSS 순서가 이기므로, 배경을 직접 주면 기본 배경을 뺀다
  const bg = /(^|\s)bg-/.test(innerClassName) ? '' : 'bg-surface'
  return (
    <div {...rest} className={`${glow} ${className}`}>
      <div className={`chamfer h-full p-px ${size === 'lg' ? '[--c:12px]' : '[--c:6px]'} ${edge}`}>
        <div className={`chamfer-inner h-full ${bg} ${innerClassName}`}>{children}</div>
      </div>
    </div>
  )
}
