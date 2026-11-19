import Button from '../../../components/ui/Button.tsx'
import Icon from '../../../components/ui/Icon.tsx'
import Switch from '../../../components/ui/Switch.tsx'

// 표현만: 화면 공유 중 웹캠을 따로 보내 시청자 화면에 겹친다(architecture §9). 위치는 아래 "배치 편집"으로 정한다.
export default function PipControl({ on, onToggle }: { on: boolean; onToggle: (on: boolean) => void }) {
  return (
    <div className='flex items-center gap-2.5'>
      <span className='flex-1 text-sm'>웹캠 같이 보내기 (PIP)</span>
      <Switch label='웹캠 PIP' checked={on} onChange={onToggle} />
    </div>
  )
}

/** 시안 Studio·StudioLayout: 영상 카드의 "배치 편집" 줄. 편집 중엔 눌린 상태(다시 누르면 취소). */
export function LayoutEditRow({ editing, onToggle }: { editing: boolean; onToggle: () => void }) {
  return (
    <div className='flex items-center gap-2.5'>
      <span className='flex-1 text-[13px] leading-[1.4] text-text-muted'>
        웹캠 · 후원 알림 · 채팅 위치
        <br />
        <span className='text-xs text-text-dim'>미리보기에서 끌어서 옮겨요</span>
      </span>
      <Button size='sm' variant={editing ? 'primary' : 'neutral'} aria-pressed={editing} onClick={onToggle}>
        <Icon name='move' className='size-3.5' />
        {editing ? '편집 중' : '배치 편집'}
      </Button>
    </div>
  )
}
