const KST = 'Asia/Seoul'

/** `11월 9일 21:42` (KST) */
export const kstStamp = (iso: string) =>
  new Date(iso).toLocaleString('ko-KR', { timeZone: KST, month: 'long', day: 'numeric', hour: '2-digit', minute: '2-digit', hour12: false })

/** 부호를 붙인 금액. 음수는 toLocaleString이 `-`를 붙인다. */
export const signed = (n: number) => `${n > 0 ? '+' : ''}${n.toLocaleString('ko-KR')}`

/** 이번 달부터 거슬러 올라간 KST 월 목록(`value`는 API의 `month=YYYY-MM`). */
export function kstMonths(count = 12) {
  const now = new Date(Date.now() + 9 * 3600_000)
  return Array.from({ length: count }, (_, i) => {
    const d = new Date(Date.UTC(now.getUTCFullYear(), now.getUTCMonth() - i, 1))
    const y = d.getUTCFullYear()
    const m = d.getUTCMonth() + 1
    return { value: `${y}-${String(m).padStart(2, '0')}`, label: `${y}년 ${m}월` }
  })
}

export const MIN_AMOUNT = 1000
export const MAX_AMOUNT = 1_000_000
