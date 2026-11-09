import Button from './Button.tsx'
import EmptyState from './EmptyState.tsx'

export type ErrorKind = 'notFound' | 'server' | 'network'

const copy: Record<ErrorKind, { title: string; description: string }> = {
  notFound: { title: '페이지를 찾을 수 없어요', description: '삭제되었거나 주소가 바뀐 것 같아요.' },
  server: { title: '서버에 문제가 생겼어요', description: '잠시 후 다시 시도해 주세요. 계속되면 아래 코드를 알려 주세요.' },
  network: { title: '네트워크 연결이 끊겼어요', description: '인터넷 연결을 확인한 뒤 다시 시도해 주세요.' },
}

type Props = {
  kind: ErrorKind
  /** 서버 오류일 때 표시(Grafana 추적용) */
  requestId?: string
  onRetry?: () => void
}

// 404·500·네트워크 끊김 공통 화면: 마스코트 + 안내 + 홈으로
export default function ErrorScreen({ kind, requestId, onRetry }: Props) {
  return (
    <EmptyState
      face='grumpy'
      title={copy[kind].title}
      description={copy[kind].description}
      action={
        <div className='flex flex-col items-center gap-4'>
          <div className='flex gap-2'>
            <Button to='/'>홈으로</Button>
            {onRetry && (
              <Button variant='secondary' onClick={onRetry}>
                다시 시도
              </Button>
            )}
          </div>
          {requestId && <p className='font-mono text-xs text-text-dim'>request_id: {requestId}</p>}
        </div>
      }
    />
  )
}
