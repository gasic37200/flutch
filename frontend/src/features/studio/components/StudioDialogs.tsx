import Button from '../../../components/ui/Button.tsx'
import ConfirmDialog from '../../../components/ui/ConfirmDialog.tsx'
import Modal from '../../../components/ui/Modal.tsx'
import type { PublisherStatus } from '../../live/hooks/usePublisher.ts'

type Props = {
  status: PublisherStatus
  onConfirmTakeover: () => void
  onCancelTakeover: () => void
  onReconnect: () => void
  endOpen: boolean
  ending: boolean
  onEnd: () => void
  onCancelEnd: () => void
}

// 기기 전환 확인 · 다른 기기로 전환됨 · 연결 끊김 · 방송 종료 확인
export default function StudioDialogs(p: Props) {
  return (
    <>
      <Modal
        open={p.status === 'confirm-takeover'}
        onClose={p.onCancelTakeover}
        title='다른 기기에서 송출 중이에요'
        footer={
          <>
            <Button variant='secondary' onClick={p.onCancelTakeover}>
              취소
            </Button>
            <Button onClick={p.onConfirmTakeover}>이 기기로 전환</Button>
          </>
        }
      >
        <p className='text-sm text-text-muted'>이 기기로 전환할까요? 전환하면 다른 기기의 송출은 끊기고, 방송은 이어져요.</p>
      </Modal>
      <Modal
        open={p.status === 'displaced'}
        onClose={() => undefined}
        title='다른 기기로 전환됐어요'
        footer={<Button to='/manage'>내 방송 관리</Button>}
      >
        <p className='text-sm text-text-muted'>다른 기기에서 이 방송을 이어서 송출하고 있어요. 이 화면에서는 송출이 멈췄고, 자동으로 다시 연결하지 않아요.</p>
      </Modal>
      <Modal
        open={p.status === 'disconnected'}
        onClose={() => undefined}
        title='송출 연결이 끊겼어요'
        footer={<Button onClick={p.onReconnect}>다시 연결</Button>}
      >
        <p className='text-sm text-text-muted'>네트워크가 불안정해 방송 서버와 연결이 끊겼어요. 60초 안에 다시 연결하면 같은 방송을 이어서 할 수 있어요.</p>
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
