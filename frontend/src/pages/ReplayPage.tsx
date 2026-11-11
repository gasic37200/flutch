import { Link, useParams, useSearchParams } from 'react-router'
import { ApiRequestError } from '../api/client.ts'
import Avatar from '../components/ui/Avatar.tsx'
import Button from '../components/ui/Button.tsx'
import { Chip } from '../components/ui/Chip.tsx'
import EmptyState from '../components/ui/EmptyState.tsx'
import ErrorScreen from '../components/ui/ErrorScreen.tsx'
import Skeleton from '../components/ui/Skeleton.tsx'
import { CATEGORY_LABEL } from '../features/broadcast/categories.ts'
import ChatHistory from '../features/replay/components/ChatHistory.tsx'
import ReplayVideo from '../features/replay/components/ReplayVideo.tsx'
import { count, kstDate } from '../features/replay/format.ts'
import { useCountView, useRecording } from '../features/replay/hooks/useRecordings.ts'
import { useNow } from '../features/watch/hooks/useWatch.ts'
import { errorKindOf } from '../hooks/errorKind.ts'
import ShellBody from '../layouts/ShellBody.tsx'

const DAY_MS = 86_400_000

export default function ReplayPage() {
  const id = Number(useParams().recordingId) || undefined
  // CLUTCH FEED 클립: ?t=시작초&d=길이초
  const sp = useSearchParams()[0]
  const start = Math.max(0, Math.floor(Number(sp.get('t')))) || 0
  const len = Math.floor(Number(sp.get('d'))) || 0
  const clip = sp.has('t') ? { start, end: len > 0 ? start + len : undefined } : undefined
  const { data: r, error, refetch } = useRecording(id)
  const countView = useCountView(id)
  const now = useNow(false)

  if (id === undefined) return <ErrorScreen kind='notFound' />
  if (error instanceof ApiRequestError && error.code === 'RECORDING_EXPIRED') {
    return <EmptyState title='보관 기간이 지난 다시보기예요' description='다시보기는 30일 동안만 보관돼요.' action={<Button to='/replay'>다른 다시보기 보기</Button>} />
  }
  if (error) {
    const e = errorKindOf(error)
    return <ErrorScreen {...e} onRetry={e.kind === 'notFound' ? undefined : () => void refetch()} />
  }
  if (!r) return <Skeleton className='aspect-video w-full max-w-4xl' />

  const daysLeft = r.expiresAt ? Math.max(0, Math.ceil((Date.parse(r.expiresAt) - now) / DAY_MS)) : null
  return (
    <ShellBody aside={<ChatHistory broadcastId={r.broadcastId} startedAt={r.startedAt} />}>
      <div className='flex flex-col gap-3.5'>
        <ReplayVideo recording={r} onPlay={countView} clip={clip} />
        <h1 className='text-base font-bold break-words sm:text-xl'>{r.title}</h1>
        <div className='flex flex-wrap items-center gap-x-3 gap-y-2 text-[13px] text-text-muted'>
          <Link to={`/users/${r.owner.id}`} className='inline-flex items-center gap-2 text-sm font-bold text-text hover:text-brand'>
            <Avatar userId={r.owner.id} nickname={r.owner.nickname} size='xs' className='size-7' />
            {r.owner.nickname}
          </Link>
          <Chip>{CATEGORY_LABEL[r.category]}</Chip>
          <span>{kstDate(r.startedAt)} 방송</span>
          <span>
            조회 <b className='font-mono text-text'>{count(r.views)}</b>
          </span>
          <span>
            최고 시청자 <b className='font-mono text-live'>{count(r.peakViewers)}</b>
          </span>
          {r.expiresAt && daysLeft !== null && (
            <span>
              {kstDate(r.expiresAt)}까지 보관 ({daysLeft}일 남음)
            </span>
          )}
        </div>
      </div>
    </ShellBody>
  )
}
