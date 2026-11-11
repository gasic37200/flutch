import { useSearchParams } from 'react-router'
import { ApiRequestError } from '../api/client.ts'
import Button from '../components/ui/Button.tsx'
import Panel from '../components/ui/Panel.tsx'
import Skeleton from '../components/ui/Skeleton.tsx'
import { faceSrc, type Face } from '../components/ui/mascot.ts'
import { usePaymentConfirm } from '../features/wallet/hooks/usePaymentConfirm.ts'
import type { ReactNode } from 'react'

const n = (v: number) => `${v.toLocaleString('ko-KR')} 비트`

function Card({ ok, face, title, children, actions }: { ok?: boolean; face: Face; title: string; children?: ReactNode; actions: ReactNode }) {
  return (
    <Panel tone={ok ? 'hero' : 'line'} className='max-w-lg' innerClassName='flex items-center gap-3.5 p-[18px]'>
      <img src={faceSrc(face)} alt='' className='size-14 shrink-0 rounded-full' />
      <div role={ok ? 'status' : 'alert'} className='flex min-w-0 flex-col items-start gap-1.5'>
        <h1 className='text-base font-bold'>{title}</h1>
        {children && <div className='text-[13px] text-text-muted'>{children}</div>}
        <div className='flex flex-wrap gap-2'>{actions}</div>
      </div>
    </Panel>
  )
}

const wallet = (
  <Button to='/wallet' variant='secondary' size='sm'>
    내 지갑 보기
  </Button>
)

/** /payments/success: 토스가 돌려준 쿼리를 서버에서 승인하고 결과를 보여 준다. */
function Success() {
  const { valid, query } = usePaymentConfirm()
  if (!valid) return <Card face='grumpy' title='결제 정보가 올바르지 않아요' actions={wallet} />
  if (query.isError) {
    const e = query.error
    const unavailable = e instanceof ApiRequestError && e.status === 502
    return (
      <Card
        face='grumpy'
        title={unavailable ? '결제 확인이 늦어지고 있어요' : '결제를 확인하지 못했어요'}
        actions={
          <>
            {unavailable && (
              <Button size='sm' onClick={() => void query.refetch()} loading={query.isFetching}>
                다시 확인
              </Button>
            )}
            {wallet}
          </>
        }
      >
        {unavailable ? '잠시 후 다시 확인해 주세요. 결제는 같은 주문으로 이어서 확인돼요.' : e instanceof ApiRequestError ? e.message : '네트워크 연결을 확인하고 다시 시도해 주세요.'}
      </Card>
    )
  }
  const r = query.data
  if (!r) return <Skeleton className='h-28 w-full max-w-lg' card />
  if (r.donation)
    return (
      <Card
        ok
        face='joy'
        title='후원했어요!'
        actions={
          <Button to={`/live/${r.donation.broadcastId}`} variant='secondary' size='sm'>
            방송으로 돌아가기
          </Button>
        }
      >
        {n(r.charged)} 충전 + {n(r.donation.amount)} 후원 · 잔액 {n(r.balance)}
      </Card>
    )
  return (
    <Card ok face='happy' title={r.notice ?? '충전했어요!'} actions={wallet}>
      {n(r.charged)}이 잔액에 들어갔어요 · 잔액 {n(r.balance)}
    </Card>
  )
}

/** /payments/fail: 토스가 실패·취소로 돌려보낸 화면. 비트는 빠져나가지 않았다. */
function Fail() {
  const [sp] = useSearchParams()
  const message = sp.get('message')
  return (
    <Card
      face='grumpy'
      title='결제가 완료되지 않았어요'
      actions={
        <Button to='/wallet' variant='secondary' size='sm'>
          다시 시도 →
        </Button>
      }
    >
      결제를 취소했거나 시간이 지났어요. 비트는 빠져나가지 않았어요.
      {message && <span className='mt-1 block text-xs text-text-dim'>{message}</span>}
    </Card>
  )
}

export default function PaymentResultPage({ result }: { result: 'success' | 'fail' }) {
  return result === 'success' ? <Success /> : <Fail />
}
