import Avatar from '../../../components/ui/Avatar.tsx'
import Button from '../../../components/ui/Button.tsx'
import EmptyState from '../../../components/ui/EmptyState.tsx'
import ErrorScreen from '../../../components/ui/ErrorScreen.tsx'
import Skeleton from '../../../components/ui/Skeleton.tsx'
import { errorKindOf } from '../../../hooks/errorKind.ts'
import { leftText, useBanList } from '../hooks/useModeration.ts'

// 스트리머 페이지 관리 탭: 지금 유효한 차단 목록(본인에게만 그린다).
export default function BanList({ ownerId }: { ownerId: number }) {
  const b = useBanList(ownerId)
  const { query } = b
  if (query.isError) return <ErrorScreen {...errorKindOf(query.error)} onRetry={() => void query.refetch()} />
  if (query.isPending) return <Skeleton className='h-40 w-full max-w-xl' />
  if (b.items.length === 0 && !query.hasNextPage) return <EmptyState title='차단한 시청자가 없어요' description='채팅 닉네임을 눌러 차단할 수 있어요.' />
  return (
    <section aria-labelledby='ban-list' className='flex max-w-xl flex-col gap-3'>
      <h2 id='ban-list' className='text-lg font-black'>
        차단 목록
      </h2>
      <ul className='flex flex-col bg-surface px-3.5 py-2'>
        {b.items.map((x) => (
          <li key={x.user.id} className='flex items-center gap-2.5 border-b border-surface-2 py-2.5 last:border-b-0'>
            <Avatar userId={x.user.id} nickname={x.user.nickname} size='sm' />
            <div className='flex min-w-0 grow flex-col gap-0.5'>
              <b className='text-sm break-all'>{x.user.nickname}</b>
              <span className='text-xs text-text-dim'>{leftText(x, b.now)}</span>
            </div>
            <Button size='sm' variant='neutral' disabled={b.busy} onClick={() => void b.unban(x.user)}>
              해제
            </Button>
          </li>
        ))}
      </ul>
      <p className='text-xs text-text-dim'>임시차단은 시간이 지나면 자동으로 풀려요.</p>
      {query.hasNextPage && (
        <div className='flex justify-center'>
          <Button variant='secondary' onClick={() => void query.fetchNextPage()} loading={query.isFetchingNextPage}>
            더 보기
          </Button>
        </div>
      )}
    </section>
  )
}
