import Button from '../../../components/ui/Button.tsx'
import EmptyState from '../../../components/ui/EmptyState.tsx'
import ErrorScreen from '../../../components/ui/ErrorScreen.tsx'
import Panel from '../../../components/ui/Panel.tsx'
import Skeleton from '../../../components/ui/Skeleton.tsx'
import { errorKindOf } from '../../../hooks/errorKind.ts'
import { leftText, useBanList } from '../hooks/useModeration.ts'

// 스트리머 페이지 관리 탭: 지금 유효한 차단 목록(본인에게만 그린다). 제목은 호출하는 쪽이 둔다.
export default function BanList({ ownerId }: { ownerId: number }) {
  const b = useBanList(ownerId)
  const { query } = b
  if (query.isError) return <ErrorScreen {...errorKindOf(query.error)} onRetry={() => void query.refetch()} />
  if (query.isPending) return <Skeleton className='h-40 w-full' card />
  if (b.items.length === 0 && !query.hasNextPage) return <EmptyState title='차단한 시청자가 없어요' description='채팅 닉네임을 눌러 차단할 수 있어요.' />
  return (
    <Panel innerClassName='flex flex-col px-3.5 py-2'>
      <ul className='flex flex-col'>
        {b.items.map((x) => (
          <li key={x.user.id} className='flex items-center gap-2.5 border-b border-surface-2 py-2.5'>
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
      {query.hasNextPage && (
        <div className='flex justify-center py-2'>
          <Button size='sm' variant='neutral' onClick={() => void query.fetchNextPage()} loading={query.isFetchingNextPage}>
            더 보기
          </Button>
        </div>
      )}
      <p className='py-2.5 text-xs text-text-dim'>임시차단은 시간이 지나면 자동으로 풀려요.</p>
    </Panel>
  )
}
