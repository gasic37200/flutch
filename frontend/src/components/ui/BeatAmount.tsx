import Icon from './Icon.tsx'

// 비트(화폐) = 라임 파동 아이콘 + 흰색 숫자(DESIGN §2). 금액에 아이콘을 빼지 않는다. unit이면 "Beat" 단위을 라임으로 붙인다.
export default function BeatAmount({ amount, unit, className = '' }: { amount: number; unit?: boolean; className?: string }) {
  return (
    <span className={`inline-flex items-center gap-1.5 font-mono font-bold text-text ${className}`}>
      <Icon name='pulse' className='size-[18px] text-brand' />
      {amount.toLocaleString('ko-KR')}
      {unit ? <span className='font-display text-[13px] text-brand'>Beat</span> : <span className='sr-only'>비트</span>}
    </span>
  )
}
