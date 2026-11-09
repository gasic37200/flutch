import { Link } from 'react-router'
import Avatar from './Avatar.tsx'
import Badge from './Badge.tsx'
import { Chip } from './Chip.tsx'
import Icon from './Icon.tsx'
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
  className?: string
}

// 라이브·다시보기 공용 카드. 썸네일이 없으면 배경 텍스처 + 카테고리 이름 + 졸린 고양이(DESIGN §5).
export default function LiveCard({ to, title, streamerId, streamerName, streamerAvatarUrl, category, thumbnailUrl, live, viewerCount, durationLabel, className }: Props) {
  return (
    <Panel interactive className={className} innerClassName='overflow-hidden'>
      <Link to={to} className='block'>
        <div className='relative aspect-video bg-surface-2'>
          {thumbnailUrl ? (
            <img src={thumbnailUrl} alt='' loading='lazy' className='size-full object-cover' />
          ) : (
            <div className='flex size-full items-center justify-center bg-cover bg-center' style={{ backgroundImage: 'url(/brand/bg-speed.jpg)' }}>
              <span className='font-display text-lg font-bold tracking-wide text-text-muted'>{category ?? 'FLUTCH'}</span>
              <img src={faceSrc('sleepy')} alt='' className='absolute right-2 bottom-2 size-8' />
            </div>
          )}
          {live && <Badge tone='live' className='absolute top-2 left-2' />}
          {live && viewerCount !== undefined && (
            <span className='absolute top-2 right-2 inline-flex items-center gap-1 rounded-xs bg-bg/70 px-1.5 font-mono text-xs text-live'>
              <Icon name='eye' className='size-3.5' />
              {viewerCount.toLocaleString('ko-KR')}
              <span className='sr-only'>명 시청 중</span>
            </span>
          )}
          {durationLabel && <span className='absolute right-2 bottom-2 rounded-xs bg-bg/70 px-1.5 font-mono text-xs'>{durationLabel}</span>}
          {category && thumbnailUrl && <Chip className='absolute bottom-2 left-2'>{category}</Chip>}
        </div>
        <div className='space-y-2 p-3'>
          <h3 className='line-clamp-2 font-semibold'>{title}</h3>
          <div className='flex items-center gap-2 text-sm text-text-muted'>
            <Avatar userId={streamerId} nickname={streamerName} src={streamerAvatarUrl} size='sm' className='size-6' />
            <span className='truncate'>{streamerName}</span>
          </div>
        </div>
      </Link>
    </Panel>
  )
}
