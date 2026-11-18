import { useState, type ReactNode } from 'react'
import { Link } from 'react-router'
import Avatar from './Avatar.tsx'
import Badge from './Badge.tsx'
import { Chip } from './Chip.tsx'
import { faceSrc } from './mascot.ts'

type Props = {
  /** 없으면 링크 없는 미리보기 카드 */
  to?: string
  title: string
  streamerId: number
  streamerName: string
  streamerAvatarUrl?: string | null
  category?: string
  /** 카테고리 칩 옆에 보일 태그(최대 2개) */
  tags?: string[]
  thumbnailUrl?: string | null
  /** 라이브 카드: LIVE 배지 + 시청자 수 */
  live?: boolean
  viewerCount?: number
  /** 다시보기 카드: 재생 시간 표시(예: 1:02:33) */
  durationLabel?: string
  /** 카드 맨 아래 보조 한 줄(다시보기: 조회수·날짜) */
  meta?: string
  /** CLUTCH FEED 클립 카드: SUPER CLUTCH 배지 */
  superClutch?: boolean
  className?: string
}

// 라이브·다시보기 공용 카드(시안 CompData D2). 썸네일은 기본 테두리 없음, 호버·키보드 포커스에서 라임 1px + 글로우.
// 썸네일이 없으면 배경 텍스처 + 졸린 고양이(DESIGN §5).
export default function LiveCard({ to, title, streamerId, streamerName, streamerAvatarUrl, category, tags = [], thumbnailUrl, live, viewerCount, durationLabel, meta, superClutch, className }: Props) {
  // 파일을 못 읽으면 썸네일 없음과 같은 기본 표시로 바꾼다
  const [failed, setFailed] = useState(false)
  const hasThumb = !!thumbnailUrl && !failed
  return (
    <Wrapper to={to} className={className}>
      <div className='transition-[filter] group-hover:drop-shadow-glow group-focus-visible:drop-shadow-glow motion-reduce:transition-none'>
        <div className='chamfer bg-transparent p-px transition-colors [--c:12px] group-hover:bg-brand group-focus-visible:bg-brand'>
          <div className='chamfer-inner relative aspect-video bg-surface-2'>
            {hasThumb ? (
              <img src={thumbnailUrl} alt='' loading='lazy' onError={() => setFailed(true)} className='size-full object-cover' />
            ) : (
              <div className='size-full bg-cover bg-center' style={{ backgroundImage: 'url(/brand/bg-speed.jpg)' }} />
            )}
            {live && <Badge tone='live' className='absolute top-2.5 left-2.5' />}
            {superClutch && <Badge tone='superClutch' className='absolute top-2.5 right-2.5' />}
            {live && viewerCount !== undefined && (
              <span className='absolute bottom-2.5 left-2.5 bg-bg/85 px-1.5 py-0.5 font-mono text-xs font-bold text-live'>
                {viewerCount.toLocaleString('ko-KR')}
                <span className='sr-only'>명 시청 중</span>
              </span>
            )}
            {durationLabel && <span className='absolute bottom-2.5 left-2.5 bg-bg/85 px-1.5 py-0.5 font-mono text-xs font-bold'>{durationLabel}</span>}
            {!hasThumb && <img src={faceSrc('sleepy')} alt='' className='absolute right-2.5 bottom-2.5 size-[26px] rounded-full opacity-85' />}
          </div>
        </div>
      </div>
      <div className='flex gap-2.5'>
        <Avatar userId={streamerId} nickname={streamerName} src={streamerAvatarUrl} size='sm' />
        <div className='flex min-w-0 flex-col gap-0.5'>
          <h3 className='line-clamp-2 text-sm leading-snug font-bold'>{title}</h3>
          <span className='truncate text-xs text-text-muted'>{streamerName}</span>
          {(category || tags.length > 0) && (
            <span className='mt-0.5 flex flex-wrap gap-1.5'>
              {category && <Chip>{category}</Chip>}
              {tags.slice(0, 2).map((t) => (
                <Chip key={t}>{t}</Chip>
              ))}
            </span>
          )}
          {meta && <span className='truncate text-xs text-text-dim'>{meta}</span>}
        </div>
      </div>
    </Wrapper>
  )
}

const Wrapper = ({ to, className = '', children }: { to?: string; className?: string; children: ReactNode }) => {
  const cls = `group flex flex-col gap-2.5 ${className}`
  return to ? (
    <Link to={to} className={`${cls} focus-visible:outline-none`}>
      {children}
    </Link>
  ) : (
    <div className={cls}>{children}</div>
  )
}
