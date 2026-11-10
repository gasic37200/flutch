import type { Donation } from '../../api/generated/Donation.ts'

/** 이 금액 이상은 큰 연출(알림·목록 강조) */
export const BIG_AMOUNT = 10_000

/** 공개형 후원은 익명이면 donor가 null이다. 소유자형은 실명이라 익명 여부는 `anonymous`로 따로 표시한다. */
export const donorName = (d: Donation) => d.donor?.nickname ?? '익명의 후원자'

export const beats = (n: number) => n.toLocaleString('ko-KR')
