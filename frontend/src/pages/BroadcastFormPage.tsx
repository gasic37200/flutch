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

  const heading = id === undefined ? '새 방송 등록' : '방송 정보 수정'
  const wrap = (body: React.ReactNode) => (
    <div className='mx-auto max-w-xl'>
      <h1 className='mb-4 text-2xl font-black sm:text-[26px]'>{heading}</h1>
      {body}
    </div>
  )

  if (id === undefined) return <BroadcastForm heading={heading} owner={me} />
  if (broadcast.isError || !Number.isInteger(id)) return wrap(<ErrorScreen {...errorKindOf(Number.isInteger(id) ? broadcast.error : undefined)} />)
  if (!broadcast.data) return wrap(<Skeleton className='h-96 w-full' />)
  // 서버도 PATCH를 403으로 막는다. 여기서는 빈 폼을 보여 주지 않을 뿐.
  if (broadcast.data.owner.id !== me.id) return wrap(<ErrorScreen kind='notFound' />)
  return <BroadcastForm key={broadcast.data.id} heading={heading} owner={me} existing={broadcast.data} />
}
