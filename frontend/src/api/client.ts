import type { ErrorBody } from './generated/ErrorBody'

/** 백엔드 에러 응답 `{error:{code,message,request_id}}`를 담는다. 화면에는 `message`와 `requestId`를 보여 준다. */
export class ApiRequestError extends Error {
  readonly status: number
  readonly code: string
  readonly requestId: string
  readonly fields?: Record<string, string>
  readonly retryAfterSec?: number

  constructor(status: number, body: ErrorBody) {
    super(body.message)
    this.name = 'ApiRequestError'
    this.status = status
    this.code = body.code
    this.requestId = body.request_id
    this.fields = body.fields
    this.retryAfterSec = body.retry_after_sec
  }
}

type Options = Omit<RequestInit, 'body'> & { json?: unknown }

/** `/api/v1` 아래 경로를 호출한다. 실패하면 `ApiRequestError`를 던진다(네트워크 끊김은 fetch의 TypeError 그대로). */
export async function api<T>(path: string, { json, headers, ...init }: Options = {}): Promise<T> {
  const res = await fetch(`/api/v1${path}`, {
    ...init,
    credentials: 'include',
    headers: json === undefined ? headers : { 'Content-Type': 'application/json', ...headers },
    body: json === undefined ? undefined : JSON.stringify(json),
  })
  if (!res.ok) throw new ApiRequestError(res.status, await errorBody(res))
  if (res.status === 204) return undefined as T
  return (await res.json()) as T
}

async function errorBody(res: Response): Promise<ErrorBody> {
  try {
    const parsed = (await res.json()) as { error?: ErrorBody }
    if (parsed.error?.code) return parsed.error
  } catch {
    // 프록시가 HTML 에러 페이지를 주는 경우 등: 아래 공통 문구로 대신한다.
  }
  return {
    code: 'INTERNAL',
    message: '일시적인 오류가 발생했습니다. 잠시 후 다시 시도해 주세요.',
    request_id: res.headers.get('x-request-id') ?? '',
  }
}
