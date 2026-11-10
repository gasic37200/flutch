import { formatDuration } from '../watch/hooks/useWatch.ts'

const kstDay = new Intl.DateTimeFormat('ko-KR', { timeZone: 'Asia/Seoul', month: 'long', day: 'numeric' })

/** `11월 8일` (KST) */
export const kstDate = (iso: string) => kstDay.format(new Date(iso))

/** 재생 시간 `1:02:33` */
export const durationLabel = (sec: number) => formatDuration(sec * 1000)

export const count = (n: number) => n.toLocaleString('ko-KR')
