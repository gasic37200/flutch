import Button from '../components/ui/Button.tsx'
import ConfirmDialog from '../components/ui/ConfirmDialog.tsx'
import EmptyState from '../components/ui/EmptyState.tsx'
import ErrorScreen from '../components/ui/ErrorScreen.tsx'
import Skeleton from '../components/ui/Skeleton.tsx'
import { useRequireLogin } from '../features/auth/hooks/useAuth.ts'
import ManageRow from '../features/broadcast/components/ManageRow.tsx'
import { deleteMessage, READY_LIMIT, useManageBroadcasts } from '../features/broadcast/hooks/useManageBroadcasts.ts'
import { errorKindOf } from '../hooks/errorKind.ts'

// 내 방송 관리: READY(송출 전) 우선, 이어서 LIVE·ENDED. 시작·수정·삭제.
export default function ManagePage() {
  const { me } = useRequireLogin()
  const m = useManageBroadcasts()
  if (!me) return null

  return (
    <div className='mx-auto max-w-3xl'>
      <div className='mb-4 flex items-center justify-between gap-3'>
        <div>
          <h1 className='font-display text-2xl font-bold tracking-wide'>내 방송 관리</h1>
          <p className='text-sm text-text-muted'>
            준비된 방송 <span className='font-mono'>{m.readyCount}/{READY_LIMIT}</span>
          </p>
        </div>
        <Button to='/broadcasts/new'>새 방송 등록</Button>
      </div>

      {m.query.isError ? (
        <ErrorScreen {...errorKindOf(m.query.error)} onRetry={() => void m.query.refetch()} />
      ) : m.query.isPending ? (
        <div className='space-y-3'>
          <Skeleton className='h-28 w-full' />
          <Skeleton className='h-28 w-full' />
        </div>
      ) : m.items.length === 0 ? (
        <EmptyState title='아직 등록한 방송이 없어요' description='방송을 등록하고 첫 클러치를 만들어 보세요.' action={<Button to='/broadcasts/new'>방송 등록</Button>} />
      ) : (
        <ul className='space-y-3'>
          {m.items.map((b) => (
            <li key={b.id}>
              <ManageRow broadcast={b} onDelete={() => m.askDelete(b)} />
            </li>
          ))}
        </ul>
      )}

      <ConfirmDialog
        open={m.target !== null}
        title='방송을 삭제할까요?'
        message={m.target ? deleteMessage(m.target) : ''}
        confirmLabel='삭제'
        loading={m.deleting}
        onConfirm={m.confirmDelete}
        onCancel={m.cancelDelete}
      />
    </div>
  )
}
