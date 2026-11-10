import { useEffect, useState } from 'react'
import Button from '../../../components/ui/Button.tsx'
import ConfirmDialog from '../../../components/ui/ConfirmDialog.tsx'
import Modal from '../../../components/ui/Modal.tsx'
import type { PublisherStatus } from '../../live/hooks/usePublisher.ts'

type Props = {
  status: PublisherStatus
  onConfirmTakeover: () => void
  onCancelTakeover: () => void
  onReconnect: () => void
  /** 연결이 끊긴 창에서 방송을 끝내려는 요청(종료 확인 창을 연다) */
  onAskEnd: () => void
  endOpen: boolean
  ending: boolean
  onEnd: () => void
  onCancelEnd: () => void
}

/** 연결이 끊긴 뒤 지난 시간(m:ss). 끊김 창이 떠 있는 동안만 센다. */
function useSinceDisconnect(active: boolean) {
  const [secs, setSecs] = useState(0)
  useEffect(() => {
    if (!active) return
    const start = Date.now()
    const id = setInterval(() => setSecs(Math.floor((Date.now() - start) / 1000)), 1000)
    return () => {
      clearInterval(id)
      setSecs(0)
    }
  }, [active])
  return `${Math.floor(secs / 60)}:${String(secs % 60).padStart(2, '0')}`
}

// 기기 전환 확인 · 다른 기기로 전환됨 · 연결 끊김(이어하기) · 방송 종료 확인
export default function StudioDialogs(p: Props) {
  const since = useSinceDisconnect(p.status === 'disconnected')
  return (
    <>
      <Modal
        open={p.status === 'confirm-takeover'}
        onClose={p.onCancelTakeover}
        title='다른 기기에서 송출 중이에요'
        footer={
          <>
            <Button variant='neutral' onClick={p.onCancelTakeover}>
              취소
            </Button>
            <Button onClick={p.onConfirmTakeover}>이 기기로 전환</Button>
          </>
        }
      >
        <p className='text-text-muted'>이 기기로 전환하면 기존 기기의 송출은 끊겨요. 방송은 그대로 이어져요.</p>
      </Modal>
      <Modal open={p.status === 'displaced'} onClose={() => undefined} title='다른 기기로 전환됐어요' footer={<Button to='/manage'>내 방송 관리</Button>}>
        <p className='text-text-muted'>다른 기기에서 이 방송을 이어서 송출하고 있어요. 이 화면에서는 송출이 멈췄고, 자동으로 다시 연결하지 않아요.</p>
      </Modal>
      <Modal
        open={p.status === 'disconnected'}
        emphasis
        onClose={() => undefined}
        title='방송을 이어서 할까요?'
        footer={
          <>
            <Button variant='neutral' onClick={p.onAskEnd}>
              종료
            </Button>
            <Button onClick={p.onReconnect}>이어서 방송</Button>
          </>
        }
      >
        <p className='text-text-muted'>
          연결이 끊긴 지 <b className='font-mono text-text'>{since}</b> · 60초 안에 돌아오면 시청자가 그대로 이어서 봐요.
        </p>
      </Modal>
      <ConfirmDialog
        open={p.endOpen}
        tone='live'
        title='방송을 종료할까요?'
        message='종료하면 시청자의 화면도 끝나요. 같은 방송을 다시 시작할 수는 없어요.'
        confirmLabel='방송 종료'
        loading={p.ending}
        onConfirm={p.onEnd}
        onCancel={p.onCancelEnd}
      />
    </>
  )
}
