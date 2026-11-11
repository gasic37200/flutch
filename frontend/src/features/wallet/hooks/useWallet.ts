import { useInfiniteQuery } from '@tanstack/react-query'
import { useState } from 'react'
import { api } from '../../../api/client.ts'
import type { LedgerEntry } from '../../../api/generated/LedgerEntry.ts'
import type { LedgerType } from '../../../api/generated/LedgerType.ts'
import type { Wallet } from '../../../api/generated/Wallet.ts'
import { useRequireLogin } from '../../auth/hooks/useAuth.ts'

export const WALLET_KEY = ['wallet']

export const LEDGER_FILTERS: { type: LedgerType | ''; label: string }[] = [
  { type: '', label: '전체' },
  { type: 'charge', label: '충전' },
  { type: 'grant', label: '테스트 지급' },
  { type: 'donation', label: '후원' },
  { type: 'clutch', label: '클러치' },
  { type: 'burst', label: 'SUPER BURST' },
  { type: 'withdraw_forfeit', label: '소멸' },
]

const TYPE_LABEL: Record<LedgerType, string> = { charge: '충전', grant: '테스트 지급', donation: '후원', clutch: '클러치', burst: 'SUPER BURST', withdraw_forfeit: '소멸' }
export const typeLabel = (t: LedgerType) => TYPE_LABEL[t]

/** 내역 한 줄의 설명. 방송 제목은 서버가 숨긴 방송이면 "삭제된 방송"으로 준다. */
export function describe(e: LedgerEntry) {
  const title = e.broadcast?.title ?? '삭제된 방송'
  switch (e.type) {
    case 'charge':
      return '토스페이먼츠 테스트 결제'
    case 'grant':
      return 'QA용 테스트 비트 지급'
    case 'clutch':
      return `${title} 클러치 ×${e.count}`
    case 'withdraw_forfeit':
      return '회원 탈퇴로 남은 비트 소멸'
    default:
      return title
  }
}

/** 내 지갑: 잔액 + 유형·월 필터 내역(커서 페이지). */
export function useWallet() {
  const { me } = useRequireLogin()
  const [type, setType] = useState<LedgerType | ''>('')
  const [month, setMonth] = useState('')
  const query = useInfiniteQuery({
    queryKey: [...WALLET_KEY, type, month],
    enabled: !!me,
    queryFn: ({ pageParam }) => {
      const p = new URLSearchParams({ limit: '20' })
      if (type) p.set('type', type)
      if (month) p.set('month', month)
      if (pageParam) p.set('cursor', pageParam)
      return api<Wallet>(`/me/wallet?${p}`)
    },
    initialPageParam: undefined as string | undefined,
    getNextPageParam: (last) => last.nextCursor ?? undefined,
  })
  const items = query.data?.pages.flatMap((p) => p.items) ?? []
  return { me, type, setType, month, setMonth, query, items, balance: query.data?.pages[0].balance ?? me?.balance ?? 0 }
}
