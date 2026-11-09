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

export default function ManageRow({ broadcast: b, onDelete }: Props) {
  return (
    <Panel innerClassName='flex flex-col gap-3 p-3 sm:flex-row sm:items-center'>
      <div className='aspect-video w-40 shrink-0'>
        <ThumbnailImage src={b.thumbnailUrl} category={CATEGORY_LABEL[b.category]} />
      </div>
      <div className='min-w-0 grow space-y-1'>
        <div className='flex items-center gap-2'>
          <Badge tone={badge[b.status]} />
          <span className='text-xs text-text-muted'>{CATEGORY_LABEL[b.category]}</span>
        </div>
        <h2 className='truncate font-semibold'>{b.title}</h2>
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
      <div className='flex flex-wrap gap-2'>
        {b.status === 'READY' && (
          <Button to={studioPath(b.id)} size='sm'>
            방송 시작
          </Button>
        )}
        {b.status === 'LIVE' && (
          <>
            <Button to={studioPath(b.id)} size='sm'>
              이어서 송출
            </Button>
            <Button to={`/live/${b.id}`} variant='secondary' size='sm'>
              보기
            </Button>
          </>
        )}
        <Button to={`/broadcasts/${b.id}/edit`} variant='secondary' size='sm'>
          수정
        </Button>
        <Button variant='outline' size='sm' onClick={onDelete}>
          삭제
        </Button>
      </div>
    </Panel>
  )
}
