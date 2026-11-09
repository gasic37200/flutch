import Button from './Button.tsx'
import Icon from './Icon.tsx'
import Modal from './Modal.tsx'

type Props = {
  open: boolean
  title: string
  message: string
  confirmLabel: string
  cancelLabel?: string
  /** live=방송 종료(빨강 버튼), warn=방송 삭제·영구 밴·탈퇴(흰 테두리 버튼 + 경고 문구) */
  tone?: 'live' | 'warn'
  loading?: boolean
  onConfirm: () => void
  onCancel: () => void
}

export default function ConfirmDialog({ open, title, message, confirmLabel, cancelLabel = '취소', tone = 'warn', loading, onConfirm, onCancel }: Props) {
  return (
    <Modal
      open={open}
      onClose={onCancel}
      title={title}
      footer={
        <>
          <Button variant='secondary' onClick={onCancel} disabled={loading}>
            {cancelLabel}
          </Button>
          <Button variant={tone === 'live' ? 'danger' : 'outline'} onClick={onConfirm} loading={loading}>
            {confirmLabel}
          </Button>
        </>
      }
    >
      <p className='flex items-start gap-2 text-sm text-text-muted'>
        {tone === 'warn' && <Icon name='alert' className='mt-0.5 size-4 text-brand' />}
        {message}
      </p>
    </Modal>
  )
}
