import { isRouteErrorResponse } from 'react-router'
import { ApiRequestError } from '../api/client.ts'
import type { ErrorKind } from '../components/ui/ErrorScreen.tsx'

/** 라우터·쿼리 에러를 에러 화면 종류로 분류한다. 에러가 없으면(= `*` 라우트) 404. */
export function errorKindOf(error: unknown): { kind: ErrorKind; requestId?: string } {
  if (error === undefined || error === null || (isRouteErrorResponse(error) && error.status === 404)) return { kind: 'notFound' }
  if (error instanceof ApiRequestError) {
    return error.status === 404 ? { kind: 'notFound' } : { kind: 'server', requestId: error.requestId || undefined }
  }
  // fetch 자체가 실패(오프라인·서버 다운)하면 TypeError
  if (error instanceof TypeError || !navigator.onLine) return { kind: 'network' }
  return { kind: 'server' }
}
