import { useState } from 'react'
import { useLocation } from 'react-router'
import type { BroadcastDetail } from '../../../api/generated/BroadcastDetail.ts'
import Button from '../../../components/ui/Button.tsx'
import Icon from '../../../components/ui/Icon.tsx'
import { useAuth } from '../../auth/hooks/useAuth.ts'
import DonateDialog from './DonateDialog.tsx'

// WatchInfo 오른쪽 끝 슬롯. LIVE가 아니거나 내 방송이면 숨기고, 비로그인은 로그인으로, 영구 밴은 안내만. 서버가 다시 검사한다.
export default function DonateButton({ broadcast }: { broadcast: BroadcastDetail }) {
  const { me, isPending } = useAuth()
  const [open, setOpen] = useState(false)
  const { pathname } = useLocation()
  if (broadcast.status !== 'LIVE' || broadcast.viewer?.isOwner || isPending) return null
  if (!me)
    return (
      <Button to={`/login?next=${encodeURIComponent(pathname)}`} size='md'>
        <Icon name='pulse' className='size-4' />
        로그인하고 후원
      </Button>
    )
  if (broadcast.viewer?.ban?.permanent) return <span className='text-xs text-text-dim'>이 채널에서는 후원할 수 없어요</span>
  return (
    <>
      <Button onClick={() => setOpen(true)} size='md'>
        <Icon name='pulse' className='size-4' />
        후원하기
      </Button>
      {open && <DonateDialog broadcast={broadcast} nickname={me.nickname} balance={me.balance} onClose={() => setOpen(false)} />}
    </>
  )
}
