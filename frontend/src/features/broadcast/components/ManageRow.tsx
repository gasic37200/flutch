import type { Broadcast } from '../../../api/generated/Broadcast.ts'
import Badge from '../../../components/ui/Badge.tsx'
import Button from '../../../components/ui/Button.tsx'
import Panel from '../../../components/ui/Panel.tsx'
import { formatDuration } from '../../watch/hooks/useWatch.ts'
import { useElapsed } from '../../studio/hooks/useElapsed.ts'
import { CATEGORY_LABEL } from '../categories.ts'
import { studioPath } from '../hooks/useManageBroadcasts.ts'

const badge = { READY: 'ready', LIVE: 'live', ENDED: 'ended' } as const

function meta(b: Broadcast, elapsed: string) {
  const parts: (string | false)[] = [CATEGORY_LABEL[b.category]]
  if (b.status === 'LIVE') parts.push(elapsed, b.record && '녹화 중')
  else if (b.status === 'READY') parts.push('대기 중', b.record && '녹화 켜짐')
  else {
    const ms = b.startedAt && b.endedAt ? Date.parse(b.endedAt) - Date.parse(b.startedAt) : undefined
    parts.push(ms !== undefined && `${formatDuration(ms)}`, b.peakViewers > 0 && `최고 ${b.peakViewers.toLocaleString('ko-KR')}명`)
  }
  return parts.filter(Boolean).join(' · ')
}

type Props = { broadcast: Broadcast; onDelete: () => void }

// 스트리머 페이지 관리 탭의 방송 한 줄: 상태 · 제목/요약 · [주 버튼] 수정 삭제. 주 버튼은 상태마다 다르다(없을 수 있음).
export default function ManageRow({ broadcast: b, onDelete }: Props) {
  const elapsed = useElapsed(b.status === 'LIVE' ? b.startedAt : null)
  const primary =
    b.status === 'READY'
      ? { label: '방송 시작', to: studioPath(b.id) }
      : b.status === 'LIVE'
        ? { label: '스튜디오 열기', to: studioPath(b.id) }
        : b.recording?.status === 'ready'
          ? { label: '다시보기', to: `/replay/${b.recording.id}` }
          : null
  return (
    <Panel innerClassName='flex flex-wrap items-center gap-x-3.5 gap-y-3 px-3.5 py-3'>
      <Badge tone={badge[b.status]} />
      <div className='flex min-w-0 flex-[1_1_240px] flex-col gap-0.5'>
        <b className='truncate text-[15px]' title={b.title}>
          {b.title}
        </b>
        <span className='text-xs text-text-dim'>{meta(b, elapsed)}</span>
      </div>
      <div className='flex gap-1.5 max-sm:w-full max-sm:[&>span]:flex-1'>
        {primary && (
          <Button to={primary.to} size='md' variant={b.status === 'ENDED' ? 'neutral' : 'primary'}>
            {primary.label}
          </Button>
        )}
        <Button to={`/broadcasts/${b.id}/edit`} variant='neutral' size='md'>
          수정
        </Button>
        <Button variant='neutral' size='md' onClick={onDelete} className='[&_span]:text-text-muted'>
          삭제
        </Button>
      </div>
    </Panel>
  )
}
