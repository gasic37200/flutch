import { useRouteError } from 'react-router'
import ErrorScreen from '../components/ui/ErrorScreen.tsx'
import { errorKindOf } from '../hooks/errorKind.ts'

// 404(`*` 라우트)·라우터 errorElement 공용. AppShell 안에서 그려진다.
export default function ErrorPage() {
  const { kind, requestId } = errorKindOf(useRouteError())
  return <ErrorScreen kind={kind} requestId={requestId} onRetry={kind === 'notFound' ? undefined : () => window.location.reload()} />
}
