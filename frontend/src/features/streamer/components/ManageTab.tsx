import Button from '../../../components/ui/Button.tsx'
import ConfirmDialog from '../../../components/ui/ConfirmDialog.tsx'
import EmptyState from '../../../components/ui/EmptyState.tsx'
import ErrorScreen from '../../../components/ui/ErrorScreen.tsx'
import Panel from '../../../components/ui/Panel.tsx'
import Skeleton from '../../../components/ui/Skeleton.tsx'
import { errorKindOf } from '../../../hooks/errorKind.ts'
import ManageRow from '../../broadcast/components/ManageRow.tsx'
import { deleteMessage, useManageBroadcasts } from '../../broadcast/hooks/useManageBroadcasts.ts'
import DonationHistory from '../../donation/components/DonationHistory.tsx'
import { useDonationStats } from '../../donation/hooks/useDonationStats.ts'
import BanList from '../../moderation/components/BanList.tsx'

const h2 = 'text-lg font-black'

// 관리 탭(본인만): 통계 카드 3 + 방송 관리 + 후원 내역 + 차단 목록을 한 화면에. 데이터는 서버가 본인에게만 준다.
export default function ManageTab({ ownerId }: { ownerId: number }) {
  const m = useManageBroadcasts()
  const stats = useDonationStats()
  const cards = [
    { label: '누적 받은 비트', amount: stats.totalReceived, lime: true },
    { label: '최근 7일', amount: stats.last7Days },
    { label: '이번 방송', amount: stats.items[0]?.total ?? 0 },
  ]
  return (
    <div className='flex flex-col gap-6'>
      <dl className='grid grid-cols-3 gap-2 sm:grid-cols-[repeat(auto-fit,minmax(200px,1fr))] sm:gap-3.5'>
        {cards.map((c) => (
          <Panel key={c.label} innerClassName='flex flex-col gap-1.5 p-3 sm:p-4'>
            <dt className='text-[13px] text-text-muted'>{c.label}</dt>
            <dd className={`font-mono text-base font-bold sm:text-2xl ${c.lime ? 'text-brand' : ''}`}>{c.amount.toLocaleString('ko-KR')}</dd>
          </Panel>
        ))}
      </dl>

      <div className='flex flex-wrap items-start gap-5'>
        <div className='flex min-w-0 flex-[999_1_560px] flex-col gap-3'>
          <h2 className={h2}>방송 관리</h2>
          {m.query.isError ? (
            <ErrorScreen {...errorKindOf(m.query.error)} onRetry={() => void m.query.refetch()} />
          ) : m.query.isPending ? (
            <Skeleton className='h-16 w-full' card />
          ) : m.items.length === 0 ? (
            <EmptyState title='아직 등록한 방송이 없어요' description='방송을 등록하고 첫 클러치를 만들어 보세요.' action={<Button to='/broadcasts/new'>방송 등록</Button>} />
          ) : (
            <ul className='flex flex-col gap-3'>
              {m.items.map((b) => (
                <li key={b.id}>
                  <ManageRow broadcast={b} onDelete={() => m.askDelete(b)} />
                </li>
              ))}
            </ul>
          )}
          <DonationHistory ownerId={ownerId} headingClass={h2} />
        </div>

        <section aria-labelledby='ban-list' className='flex min-w-[min(300px,100%)] flex-[1_1_300px] flex-col gap-3'>
          <h2 id='ban-list' className={h2}>
            차단 목록
          </h2>
          <BanList ownerId={ownerId} />
        </section>
      </div>

      <ConfirmDialog
        open={m.target !== null}
        title='방송을 삭제할까요?'
        message={m.target ? `「${m.target.title}」 ${deleteMessage(m.target)}` : ''}
        confirmLabel='삭제'
        loading={m.deleting}
        onConfirm={m.confirmDelete}
        onCancel={m.cancelDelete}
      />
    </div>
  )
}
