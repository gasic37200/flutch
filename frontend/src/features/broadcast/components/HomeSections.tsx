import { Link } from 'react-router'
import type { Broadcast } from '../../../api/generated/Broadcast.ts'
import Avatar from '../../../components/ui/Avatar.tsx'
import Badge from '../../../components/ui/Badge.tsx'
import { Chip } from '../../../components/ui/Chip.tsx'
import Icon from '../../../components/ui/Icon.tsx'
import Panel from '../../../components/ui/Panel.tsx'
import { CATEGORY_LABEL } from '../categories.ts'
import ThumbnailImage from './ThumbnailImage.tsx'

const count = (n: number) => n.toLocaleString('ko-KR')

/** 홈 메인 라이브: 목록 1위(시청자 수)를 크게. 게이지 1위 선정·CLUTCH TIME 패널은 T29. */
export function MainLive({ broadcast: b }: { broadcast: Broadcast }) {
  return (
    <Panel tone='hero' innerClassName='overflow-hidden'>
      <Link to={`/live/${b.id}`} className='block' aria-label={`${b.owner.nickname} 라이브 보기`}>
        <div className='relative aspect-video'>
          <ThumbnailImage src={b.thumbnailUrl} category={CATEGORY_LABEL[b.category]} />
          <div className='absolute inset-0 bg-linear-to-b from-bg/15 to-bg/55' />
          <Badge tone='live' className='absolute top-3 left-3 sm:top-4 sm:left-4' />
        </div>
        <div className='flex items-center gap-3.5 px-4 py-3 sm:px-[18px] sm:py-4'>
          <Avatar userId={b.owner.id} nickname={b.owner.nickname} size='sm' className='sm:size-12' />
          <div className='flex min-w-0 grow flex-col gap-1.5'>
            <div className='flex flex-wrap items-center gap-2'>
              <b className='text-[17px]'>{b.owner.nickname}</b>
              <Chip>{CATEGORY_LABEL[b.category]}</Chip>
              {b.tags.length > 0 && <span className='text-xs text-text-dim'>{b.tags.map((t) => `#${t}`).join(' ')}</span>}
            </div>
            <p className='truncate'>{b.title}</p>
          </div>
          <span className='inline-flex items-center gap-1.5 font-mono font-bold whitespace-nowrap text-live'>
            <Icon name='user' className='size-4' />
            {count(b.viewerCount)}명
          </span>
        </div>
      </Link>
    </Panel>
  )
}

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
