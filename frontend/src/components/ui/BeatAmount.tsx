import Icon from './Icon.tsx'

// 비트(화폐) = 라임 파동 아이콘 + 흰색 숫자(DESIGN §2). 금액에 아이콘을 빼지 않는다.
export default function BeatAmount({ amount, className = '' }: { amount: number; className?: string }) {
  return (
    <span className={`inline-flex items-center gap-1 font-mono font-semibold text-text ${className}`}>
      <Icon name='pulse' className='size-4 text-brand' />
      {amount.toLocaleString('ko-KR')}
      <span className='sr-only'>비트</span>
    </span>
  )
}
