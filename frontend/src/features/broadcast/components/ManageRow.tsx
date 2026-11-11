import { Link } from 'react-router'
import type { Broadcast } from '../../../api/generated/Broadcast.ts'
import Badge from '../../../components/ui/Badge.tsx'
import Button from '../../../components/ui/Button.tsx'
import { Chip } from '../../../components/ui/Chip.tsx'
import Panel from '../../../components/ui/Panel.tsx'
import { CATEGORY_LABEL } from '../categories.ts'
import { studioPath } from '../hooks/useManageBroadcasts.ts'
import ThumbnailImage from './ThumbnailImage.tsx'

const badge = { READY: 'ready', LIVE: 'live', ENDED: 'ended' } as const

type Props = { broadcast: Broadcast; onDelete: () => void }

// 버튼 영역은 폭 고정(주 · 수정 · 삭제 3칸)이라 제목 길이와 상관없이 열이 맞는다. 상태에 주 버튼이 없으면 첫 칸이 빈다.
export default function ManageRow({ broadcast: b, onDelete }: Props) {
  const primary =
    b.status === 'READY' ? { label: '방송 시작', to: studioPath(b.id) } : b.status === 'LIVE' ? { label: '이어서 송출', to: studioPath(b.id) } : null
  return (
    <Panel innerClassName='flex flex-col gap-3 p-3 sm:flex-row sm:items-center'>
      {/* 라이브 중에는 썸네일이 시청 화면 링크(별도 "보기" 버튼 대신) */}
      <div className='aspect-video w-full shrink-0 sm:w-40'>
        {b.status === 'LIVE' ? (
          <Link to={`/live/${b.id}`} aria-label={`${b.title} 보기`} className='block size-full'>
            <ThumbnailImage src={b.thumbnailUrl} category={CATEGORY_LABEL[b.category]} />
          </Link>
        ) : (
          <ThumbnailImage src={b.thumbnailUrl} category={CATEGORY_LABEL[b.category]} />
        )}
      </div>
      <div className='min-w-0 grow space-y-1'>
        <div className='flex items-center gap-2'>
          <Badge tone={badge[b.status]} />
          <span className='text-xs text-text-muted'>{CATEGORY_LABEL[b.category]}</span>
        </div>
        <h2 className='truncate font-semibold' title={b.title}>
          {b.title}
        </h2>
        {b.tags.length > 0 && (
          <ul className='flex flex-wrap gap-1' aria-label='태그'>
            {b.tags.map((t) => (
              <li key={t}>
                <Chip>#{t}</Chip>
              </li>
            ))}
          </ul>
        )}
      </div>
      <div className='grid shrink-0 grid-cols-3 gap-2 sm:w-[312px] [&>span]:min-w-0 [&_a]:whitespace-nowrap'>
        {primary ? (
          <Button to={primary.to} size='md'>
            {primary.label}
          </Button>
        ) : (
          <span aria-hidden className='hidden sm:block' />
        )}
        <Button to={`/broadcasts/${b.id}/edit`} variant='secondary' size='md'>
          수정
        </Button>
        <Button variant='outline' size='md' onClick={onDelete}>
          삭제
        </Button>
      </div>
    </Panel>
  )
}
