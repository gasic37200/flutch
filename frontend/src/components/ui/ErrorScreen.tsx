import Button from './Button.tsx'
import Icon from './Icon.tsx'
import Panel from './Panel.tsx'
import { faceSrc } from './mascot.ts'

export type ErrorKind = 'notFound' | 'server' | 'network'

const copy: Record<ErrorKind, { code?: string; title: string; description: string[]; face: 'grumpy' | 'sleepy' }> = {
  notFound: { code: '404', title: '찾는 페이지가 없어요', description: ['주소가 바뀌었거나', '삭제된 방송일 수 있어요.'], face: 'grumpy' },
  server: { code: '500', title: '잠시 문제가 생겼어요', description: ['잠시 후 다시 시도해 주세요.', '문제가 계속되면 아래 번호를 알려주세요.'], face: 'grumpy' },
  network: { title: '인터넷 연결이 끊겼어요', description: ['연결되면 자동으로 다시 불러올게요.'], face: 'sleepy' },
}

type Props = {
  kind: ErrorKind
  /** 서버 오류일 때 표시(Grafana 추적용) */
  requestId?: string
  onRetry?: () => void
}

// 404·500·네트워크 끊김 공통 화면: 배경 텍스처 위에 마스코트 + 안내 + 홈으로. 네트워크는 연결이 돌아오면 ErrorPage가 다시 불러온다.
export default function ErrorScreen({ kind, requestId, onRetry }: Props) {
  const c = copy[kind]
  return (
    <Panel className='mx-auto max-w-md' innerClassName='relative overflow-hidden bg-bg'>
      <div className='absolute inset-0 bg-cover bg-center' style={{ backgroundImage: 'url(/brand/bg-speed.jpg)' }} aria-hidden />
      <div className='absolute inset-0 bg-bg/85' aria-hidden />
      <div className='relative flex flex-col items-center gap-4 p-8 text-center'>
        {c.code && <span className='font-display text-6xl leading-none font-bold text-brand italic'>{c.code}</span>}
        {kind === 'network' && <Icon name='wifiOff' className='size-16 text-brand' />}
        <img src={faceSrc(c.face)} alt='' className='size-30 rounded-full' />
        <h2 className='text-[22px] font-bold'>{c.title}</h2>
        <p className='text-sm leading-relaxed text-text-muted'>
          {c.description.map((line, i) => (
            <span key={line}>
              {i > 0 && <br />}
              {line}
            </span>
          ))}
        </p>
        {kind === 'server' && requestId && <code className='chamfer bg-surface-2 px-3 py-2 font-mono text-[13px] [--c:6px]'>request_id: {requestId}</code>}
        {kind === 'network' ? (
          <span className='inline-flex items-center gap-2 text-[13px] text-text-dim'>
            <span className='size-2 rounded-full bg-brand' aria-hidden />
            다시 연결하는 중…
          </span>
        ) : (
          <div className='flex gap-2'>
            {onRetry && (
              <Button size='lg' onClick={onRetry}>
                다시 시도
              </Button>
            )}
            <Button size='lg' variant={onRetry ? 'neutral' : 'primary'} to='/'>
              홈으로
            </Button>
          </div>
        )}
      </div>
    </Panel>
  )
}
