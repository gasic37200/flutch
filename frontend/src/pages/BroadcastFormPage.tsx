import { useParams } from 'react-router'
import ErrorScreen from '../components/ui/ErrorScreen.tsx'
import Skeleton from '../components/ui/Skeleton.tsx'
import { useRequireLogin } from '../features/auth/hooks/useAuth.ts'
import BroadcastForm from '../features/broadcast/components/BroadcastForm.tsx'
import { useBroadcast } from '../features/broadcast/hooks/useBroadcasts.ts'
import { errorKindOf } from '../hooks/errorKind.ts'

// 방송 등록(/broadcasts/new)과 수정(/broadcasts/:broadcastId/edit) 공용 화면
export default function BroadcastFormPage() {
  const { me } = useRequireLogin()
  const param = useParams().broadcastId
  const id = param === undefined ? undefined : Number(param)
  const broadcast = useBroadcast(Number.isInteger(id) ? id : undefined)
  if (!me) return null

  let body
  if (id === undefined) {
    body = <BroadcastForm />
  } else if (broadcast.isError || !Number.isInteger(id)) {
    body = <ErrorScreen {...errorKindOf(Number.isInteger(id) ? broadcast.error : undefined)} />
  } else if (!broadcast.data) {
    body = <Skeleton className='h-96 w-full' />
  } else if (broadcast.data.owner.id !== me.id) {
    // 서버도 PATCH를 403으로 막는다. 여기서는 빈 폼을 보여 주지 않을 뿐.
    body = <ErrorScreen kind='notFound' />
  } else {
    body = <BroadcastForm key={broadcast.data.id} existing={broadcast.data} />
  }

  return (
    <div className='mx-auto max-w-xl'>
      <h1 className='mb-4 font-display text-2xl font-bold tracking-wide'>{id === undefined ? '방송 등록' : '방송 정보 수정'}</h1>
      {body}
    </div>
  )
}
