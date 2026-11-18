import Button from '../../../components/ui/Button.tsx'
import Icon from '../../../components/ui/Icon.tsx'
import { MAX_CHAT, type useChatComposer } from '../hooks/useChatComposer.ts'

const box = 'flex min-h-11 items-center gap-2 bg-surface-dim px-3 text-sm text-text-dim'

export default function ChatComposer({ c }: { c: ReturnType<typeof useChatComposer> }) {
  if (c.mode === 'pending') return null
  if (c.mode === 'guest')
    return (
      <div className='flex flex-col gap-2.5 border-t border-surface-2 px-4 py-3.5'>
        <p className='text-center text-sm text-text-muted'>로그인하고 채팅에 참여하세요</p>
        <Button to={c.loginTo} size='lg' className='w-full'>
          로그인하고 채팅하기
        </Button>
      </div>
    )
  if (c.mode === 'closed' || c.mode === 'banned')
    return (
      <div className='border-t border-surface-2 px-4 py-3'>
        <p role='status' className={`chamfer [--c:6px] ${box}`}>
          <Icon name={c.mode === 'banned' && !c.permanent ? 'clock' : 'lock'} className='size-4' />
          {c.mode === 'closed' ? '방송 중에만 채팅할 수 있어요' : c.banText}
        </p>
      </div>
    )
  return (
    <div className='flex flex-col gap-1.5 border-t border-surface-2 px-4 py-3'>
      <div className='focus-within:drop-shadow-field'>
        <div className='chamfer bg-surface-2 p-px [--c:6px] focus-within:bg-brand'>
          <div className='chamfer-inner flex h-11 items-center gap-1 bg-surface-2 pr-1 pl-3'>
            <input
              aria-label='채팅 입력'
              placeholder='채팅을 입력하세요'
              maxLength={MAX_CHAT}
              value={c.text}
              onChange={(e) => c.setText(e.target.value)}
              onKeyDown={(e) => {
                // 한글 조합 중 Enter는 글자 확정이므로 전송하지 않는다
                if (e.key !== 'Enter' || e.nativeEvent.isComposing || e.keyCode === 229) return
                e.preventDefault()
                void c.submit()
              }}
              className='min-w-0 grow bg-transparent text-sm text-text outline-none placeholder:text-text-dim'
            />
            <span className='shrink-0 font-mono text-xs text-text-dim' aria-hidden>
              {c.text.length}/{MAX_CHAT}
            </span>
            <button
              type='button'
              aria-label='보내기'
              onClick={() => void c.submit()}
              disabled={!c.canSend}
              className='flex size-9 items-center justify-center text-brand disabled:text-text-dim'
            >
              <Icon name='send' className='size-[18px]' />
            </button>
          </div>
        </div>
      </div>
      <div className='flex items-start gap-2 text-xs'>
        {c.error ? (
          <p role='alert' className='flex items-center gap-1 text-text'>
            <Icon name='alert' className='size-3.5' />
            {c.error}
          </p>
        ) : (
          <span className='text-text-dim max-[600px]:hidden'>링크는 클릭되지 않아요 · 같은 메시지는 30초에 3번까지</span>
        )}
      </div>
    </div>
  )
}
