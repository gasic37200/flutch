import { useSearchParams } from 'react-router'
import { ApiRequestError } from '../api/client.ts'
import Button from '../components/ui/Button.tsx'
import { Link } from 'react-router'
import Panel from '../components/ui/Panel.tsx'
import Skeleton from '../components/ui/Skeleton.tsx'
import { emoteSrc, type Emote } from '../components/ui/mascot.ts'
import { paymentRetryPath, usePaymentConfirm } from '../features/wallet/hooks/usePaymentConfirm.ts'
import type { ReactNode } from 'react'

// 시안: 결과 카드의 이동은 버튼이 아니라 라임색 글자 링크 + 화살표
const ActionLink = ({ to, children }: { to: string; children: ReactNode }) => (
  <Link to={to} className='text-brand hover:text-brand-hover'>
    {children} →
  </Link>
)

const n = (v: number) => `${v.toLocaleString('ko-KR')} Beat`

function Card({ ok, emote, title, children, actions }: { ok?: boolean; emote: Emote; title: string; children?: ReactNode; actions: ReactNode }) {
  return (
    <Panel tone={ok ? 'hero' : 'line'} className='max-w-[427px]' innerClassName='flex items-center gap-3.5 p-[18px]'>
      <img src={emoteSrc(emote)} alt='' className='h-14 w-auto shrink-0 object-contain' />
      <div role={ok ? 'status' : 'alert'} className='flex min-w-0 flex-col items-start gap-1'>
        <h1 className='text-base font-bold'>{title}</h1>
        {children && <div className='text-[13px] break-keep text-text-muted'>{children}</div>}
        <div className='flex flex-wrap gap-3 text-[13px]'>{actions}</div>
      </div>
    </Panel>
  )
}

const wallet = (
  <ActionLink to='/wallet'>내 지갑 보기</ActionLink>
)

/** /payments/success: 토스가 돌려준 쿼리를 서버에서 승인하고 결과를 보여 준다. */
function Success() {
  const { valid, query } = usePaymentConfirm()
  if (!valid) return <Card emote='sweat' title='결제 정보가 올바르지 않아요' actions={wallet} />
  if (query.isError) {
    const e = query.error
    // 4xx는 서버가 토스 승인 전에 막았거나 토스가 승인을 거절한 경우라 돈이 나가지 않았다(주문은 pending으로 남음, api.md 결제 절).
    // 502·네트워크 끊김은 승인됐을 수도 있어 같은 주문으로 다시 확인한다(멱등).
    if (e instanceof ApiRequestError && e.status < 500) return <NotCompleted reason={e.message}>결제가 승인되지 않아 비트는 빠져나가지 않았어요.</NotCompleted>
    return (
      <Card
        emote='sweat'
        title='결제 확인이 늦어지고 있어요'
        actions={
          <>
            <Button size='sm' onClick={() => void query.refetch()} loading={query.isFetching}>
              다시 확인
            </Button>
            {wallet}
          </>
        }
      >
        잠시 후 다시 확인해 주세요. 결제는 같은 주문으로 이어서 확인돼요.
      </Card>
    )
  }
  const r = query.data
  if (!r) return <Skeleton className='h-28 w-full max-w-lg' card />
  if (r.donation)
    return (
      <Card
        ok
        emote='excited'
        title='후원했어요!'
        actions={
          <ActionLink to={`/live/${r.donation.broadcastId}`}>방송으로 돌아가기</ActionLink>
        }
      >
        {n(r.charged)} 충전 + {n(r.donation.amount)} 후원 · 잔액 {n(r.balance)}
      </Card>
    )
  return (
    <Card ok={!r.notice} emote='excited' title={r.notice ?? '충전했어요!'} actions={wallet}>
      {n(r.charged)}이 잔액에 들어갔어요 · 잔액 {n(r.balance)}
    </Card>
  )
}

/** 결제가 끝나지 않은 화면(토스 실패·취소, 서버 승인 거절). 돈은 빠져나가지 않았고, 결제를 시작한 화면에서 다시 시도한다. */
function NotCompleted({ reason, children }: { reason?: string | null; children: ReactNode }) {
  return (
    <Card
      emote='sweat'
      title='결제가 완료되지 않았어요'
      actions={
        <ActionLink to={paymentRetryPath()}>다시 시도</ActionLink>
      }
    >
      {children}
      {reason && <span className='mt-1 block text-xs text-text-dim'>{reason}</span>}
    </Card>
  )
}

/** /payments/fail: 토스가 실패·취소로 돌려보낸 화면. */
function Fail() {
  const [sp] = useSearchParams()
  return <NotCompleted reason={sp.get('message')}>결제를 취소했거나 시간이 지났어요. 비트는 빠져나가지 않았어요.</NotCompleted>
}

export default function PaymentResultPage({ result }: { result: 'success' | 'fail' }) {
  return result === 'success' ? <Success /> : <Fail />
}
