import { Link } from 'react-router'
import Avatar from './Avatar.tsx'
import Badge from './Badge.tsx'
import Panel from './Panel.tsx'
import { faceSrc } from './mascot.ts'

type Props = {
  to: string
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
  className?: string
}

// 라이브·다시보기 공용 카드. 썸네일이 없으면 배경 텍스처 + 카테고리 이름 + 졸린 고양이(DESIGN §5).
export default function LiveCard({ to, title, streamerId, streamerName, streamerAvatarUrl, category, thumbnailUrl, live, viewerCount, durationLabel, meta, className }: Props) {
  return (
    <Panel interactive className={className} innerClassName='overflow-hidden'>
      <Link to={to} className='block'>
        <div className='relative aspect-video bg-surface-2'>
          {thumbnailUrl ? (
            <img src={thumbnailUrl} alt='' loading='lazy' className='size-full object-cover' />
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
          {category && <span className='absolute bottom-2 left-2 font-display text-[13px] font-bold tracking-wide'>{category}</span>}
          {durationLabel ? (
            <span className='absolute right-2 bottom-2 rounded-xs bg-bg/70 px-1.5 font-mono text-xs'>{durationLabel}</span>
          ) : (
            !thumbnailUrl && <img src={faceSrc('sleepy')} alt='' className='absolute right-2 bottom-2 size-[26px] rounded-full opacity-85' />
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
      </Link>
    </Panel>
  )
}
