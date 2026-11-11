import type { BroadcastDetail } from '../../../api/generated/BroadcastDetail.ts'
import Button from '../../../components/ui/Button.tsx'
import { ChoiceChip } from '../../../components/ui/Chip.tsx'
import Field from '../../../components/ui/Field.tsx'
import Modal from '../../../components/ui/Modal.tsx'
import { FormAlert } from '../../auth/components/FormParts.tsx'
import PaymentStep from '../../wallet/components/PaymentStep.tsx'
import { beats } from '../format.ts'
import { DONATE_PRESETS, MAX_MESSAGE, useDonate } from '../hooks/useDonate.ts'

type Props = { broadcast: BroadcastDetail; nickname: string; balance: number; onClose: () => void }

// 후원 창: 금액·표시 이름·메시지 → 잔액 후원, 잔액이 모자라면 "충전하고 후원하기"(결제위젯). 열 때마다 마운트해 새로 시작한다.
export default function DonateDialog({ broadcast, nickname, balance, onClose }: Props) {
  const d = useDonate(broadcast.id, balance, onClose)
  const title = `${broadcast.owner.nickname}님에게 후원`
  if (d.order)
    return (
      <Modal open onClose={onClose} title='결제'>
        <PaymentStep order={d.order} onBack={d.back} backLabel='금액 다시 정하기' />
      </Modal>
    )
  return (
    <Modal open onClose={onClose} title={title}>
      <form
        className='flex flex-col gap-3.5'
        onSubmit={(e) => {
          e.preventDefault()
          d.submit()
        }}
      >
        <div role='group' aria-label='후원 금액' className='flex flex-wrap gap-2'>
          {DONATE_PRESETS.map((n) => (
            <ChoiceChip key={n} selected={!d.custom && d.preset === n} onClick={() => d.selectPreset(n)}>
              {beats(n)}
            </ChoiceChip>
          ))}
        </div>
        <Field
          label='직접 입력 (1,000 ~ 1,000,000)'
          inputMode='numeric'
          placeholder='비트'
          maxLength={9}
          value={d.custom ? beats(Number(d.custom)) : ''}
          onChange={(e) => d.setCustom(e.target.value)}
          error={d.fields.amount}
        />
        <label className='flex items-center gap-2'>
          <input type='checkbox' checked={d.anonymous} onChange={(e) => d.setAnonymous(e.target.checked)} className='size-[18px] accent-brand' />
          <span>
            익명으로 후원 <span className='text-text-dim'>(표시 이름: {d.anonymous ? '익명의 후원자' : nickname})</span>
          </span>
        </label>
        <Field
          multiline
          label={`메시지 (선택, ${MAX_MESSAGE}자)`}
          rows={2}
          maxLength={MAX_MESSAGE}
          counter
          value={d.message}
          onChange={(e) => d.setMessage(e.target.value)}
          error={d.fields.message}
        />
        <div className='flex justify-between'>
          <span className='text-text-muted'>내 잔액</span>
          <b className='font-mono'>{beats(balance)} 비트</b>
        </div>
        <FormAlert message={d.error} />
        <Button type='submit' size='lg' loading={d.sending} disabled={!d.valid}>
          {!d.valid ? '후원하기' : d.needsCharge ? `${beats(d.chargeAmount)}비트 충전하고 후원하기` : `${beats(d.amount)}비트 후원하기`}
        </Button>
        {d.needsCharge && d.chargeAmount > d.shortage && <p className='text-xs text-text-dim'>최소 충전 단위(1,000비트) 때문에 남는 비트는 잔액에 남아요.</p>}
        <p className='text-xs text-text-dim'>테스트 모드 충전은 환불을 지원하지 않아요.</p>
      </form>
    </Modal>
  )
}
