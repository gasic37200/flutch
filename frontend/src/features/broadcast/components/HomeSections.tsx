import { Link } from 'react-router'
import type { Broadcast } from '../../../api/generated/Broadcast.ts'
import Avatar from '../../../components/ui/Avatar.tsx'
import Badge from '../../../components/ui/Badge.tsx'
import { CATEGORY_LABEL } from '../categories.ts'
import ThumbnailImage from './ThumbnailImage.tsx'

const count = (n: number) => n.toLocaleString('ko-KR')

/** 추천 라이브 채널: 썸네일 152px + 정보 가로 카드(시안: 패널 없이 썸네일만 호버 라임 테두리). */
export function RecommendedCard({ broadcast: b }: { broadcast: Broadcast }) {
  return (
    <Link to={`/live/${b.id}`} className='group flex min-w-0 items-center gap-3 focus-visible:outline-none'>
      <div className='shrink-0 basis-[152px] transition-[filter] group-hover:drop-shadow-glow group-focus-visible:drop-shadow-glow motion-reduce:transition-none'>
        <div className='chamfer bg-transparent p-px transition-colors [--c:12px] group-hover:bg-brand group-focus-visible:bg-brand'>
          <div className='chamfer-inner relative aspect-video overflow-hidden bg-surface-2'>
            <ThumbnailImage src={b.thumbnailUrl} category={CATEGORY_LABEL[b.category]} />
            <Badge tone='live' className='absolute top-1.5 left-1.5 text-[11px]' />
            <span className='absolute bottom-1.5 left-1.5 bg-bg/85 px-1.5 py-px font-mono text-[11px] font-bold text-live'>
              {count(b.viewerCount)}
              <span className='sr-only'>명 시청 중</span>
            </span>
          </div>
        </div>
      </div>
      <div className='flex min-w-0 flex-col gap-1'>
        <div className='flex items-center gap-1.5'>
          <Avatar userId={b.owner.id} nickname={b.owner.nickname} size='xs' />
          <b className='truncate text-sm'>{b.owner.nickname}</b>
        </div>
        <span className='line-clamp-2 text-[13px] leading-[1.4] group-hover:text-brand-hover'>{b.title}</span>
        <span className='text-xs text-text-dim'>{CATEGORY_LABEL[b.category]}</span>
      </div>
    </Link>
  )
}
