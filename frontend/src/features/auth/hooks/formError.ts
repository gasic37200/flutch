import { ApiRequestError } from '../../../api/client.ts'

export type FormError = { fields: Record<string, string>; message?: string }

/** 서버 에러를 필드별 문구(`fields`)와 폼 전체 문구(`message`)로 나눈다. `known`에 없는 필드의 문구는 message로 모은다. */
export function formError(e: unknown, known: string[]): FormError {
  if (!(e instanceof ApiRequestError)) return { fields: {}, message: '네트워크 연결을 확인하고 다시 시도해 주세요.' }
  const entries = Object.entries(e.fields ?? {})
  const fields = Object.fromEntries(entries.filter(([k]) => known.includes(k)))
  if (e.code === 'VALIDATION_FAILED') return { fields, message: entries.find(([k]) => !known.includes(k))?.[1] }
  // 요청 제한(429)은 서버 문구에 시간이 없을 수 있어 남은 시간을 덧붙인다
  const wait = e.retryAfterSec && e.code === 'RATE_LIMITED' ? ` ${Math.ceil(e.retryAfterSec / 60)}분 후 다시 시도해 주세요.` : ''
  return { fields, message: e.message + wait }
}
