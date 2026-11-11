import { useState, type ReactNode } from 'react'
import { Link } from 'react-router'
import Avatar from './Avatar.tsx'
import Badge from './Badge.tsx'
import Panel from './Panel.tsx'
import { faceSrc } from './mascot.ts'

type Props = {
  /** 없으면 링크 없는 미리보기 카드 */
  to?: string
  title: string
  streamerId: number
  streamerName: string
  streamerAvatarUrl?: string | null
  category?: string
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

// 라이브·다시보기 공용 카드. 썸네일이 없으면 배경 텍스처 + 카테고리 이름 + 졸린 고양이(DESIGN §5).
export default function LiveCard({ to, title, streamerId, streamerName, streamerAvatarUrl, category, thumbnailUrl, live, viewerCount, durationLabel, meta, superClutch, className }: Props) {
  // 파일을 못 읽으면 썸네일 없음과 같은 기본 표시로 바꾼다
  const [failed, setFailed] = useState(false)
  const hasThumb = !!thumbnailUrl && !failed
  return (
    <Panel interactive={!!to} className={className} innerClassName='overflow-hidden'>
      <Wrapper to={to}>
        <div className='relative aspect-video bg-surface-2'>
          {hasThumb ? (
            <img src={thumbnailUrl} alt='' loading='lazy' onError={() => setFailed(true)} className='size-full object-cover' />
          ) : (
            <div className='size-full bg-cover bg-center' style={{ backgroundImage: 'url(/brand/bg-speed.jpg)' }} />
          )}
          <div className='absolute inset-0 bg-linear-to-b from-transparent from-40% to-bg/80' />
          {live && <Badge tone='live' className='absolute top-2 left-2' />}
          {live && viewerCount !== undefined && (
            <span className='absolute top-2 right-2 bg-bg/80 px-1.5 py-0.5 font-mono text-xs'>
              {viewerCount.toLocaleString('ko-KR')}
              <span className='sr-only'>명 시청 중</span>
            </span>
          )}
          {superClutch && <Badge tone='superClutch' className='absolute top-2 right-2' />}
          {category && <span className='absolute bottom-2 left-2 font-display text-[13px] font-bold tracking-wide'>{category}</span>}
          {durationLabel ? (
            <span className='absolute right-2 bottom-2 rounded-xs bg-bg/70 px-1.5 font-mono text-xs'>{durationLabel}</span>
          ) : (
            !hasThumb && <img src={faceSrc('sleepy')} alt='' className='absolute right-2 bottom-2 size-[26px] rounded-full opacity-85' />
          )}
        </div>
        <div className='flex flex-col gap-2 px-3.5 py-3'>
          <h3 className='truncate text-[15px] font-bold'>{title}</h3>
          <div className='flex items-center gap-1.5 text-[13px] text-text-muted'>
            <Avatar userId={streamerId} nickname={streamerName} src={streamerAvatarUrl} size='xs' />
            <span className='truncate'>{streamerName}</span>
          </div>
          {meta && <span className='truncate text-xs text-text-dim'>{meta}</span>}
        </div>
      </Wrapper>
    </Panel>
  )
}

const Wrapper = ({ to, children }: { to?: string; children: ReactNode }) => (to ? <Link to={to} className='block'>{children}</Link> : <div>{children}</div>)
