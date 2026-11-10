import { Link } from 'react-router'
import type { Broadcast } from '../../../api/generated/Broadcast.ts'
import Avatar from '../../../components/ui/Avatar.tsx'
import Badge from '../../../components/ui/Badge.tsx'
import Panel from '../../../components/ui/Panel.tsx'
import { CATEGORY_LABEL } from '../categories.ts'
import ThumbnailImage from './ThumbnailImage.tsx'

const count = (n: number) => n.toLocaleString('ko-KR')

/** 추천 라이브 채널(데스크톱): 썸네일 120px + 정보 가로 카드. */
export function RecommendedCard({ broadcast: b }: { broadcast: Broadcast }) {
  return (
    <Panel interactive innerClassName='p-3'>
      <Link to={`/live/${b.id}`} className='flex items-center gap-3'>
        <div className='chamfer relative aspect-video w-[120px] shrink-0 overflow-hidden [--c:6px]'>
          <ThumbnailImage src={b.thumbnailUrl} category={CATEGORY_LABEL[b.category]} />
          <Badge tone='live' className='absolute top-1.5 left-1.5 text-[11px]' />
        </div>
        <div className='flex min-w-0 flex-col gap-1'>
          <div className='flex items-center gap-1.5'>
            <Avatar userId={b.owner.id} nickname={b.owner.nickname} size='xs' />
            <b className='truncate text-sm'>{b.owner.nickname}</b>
          </div>
          <span className='truncate text-[13px] text-text-muted'>{b.title}</span>
          <div className='flex gap-2.5 text-xs'>
            <span className='text-text-dim'>{CATEGORY_LABEL[b.category]}</span>
            <b className='font-mono text-live'>{count(b.viewerCount)}명</b>
          </div>
        </div>
      </Link>
    </Panel>
  )
}
