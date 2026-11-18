import { useParams } from 'react-router'
import Button from '../components/ui/Button.tsx'
import EmptyState from '../components/ui/EmptyState.tsx'
import ErrorScreen from '../components/ui/ErrorScreen.tsx'
import Skeleton from '../components/ui/Skeleton.tsx'
import StudioTopBar from '../features/studio/components/StudioTopBar.tsx'
import { useRequireLogin } from '../features/auth/hooks/useAuth.ts'
import BroadcastForm from '../features/broadcast/components/BroadcastForm.tsx'
import { useBroadcast } from '../features/broadcast/hooks/useBroadcasts.ts'
import { errorKindOf } from '../hooks/errorKind.ts'

// 방송 등록(/broadcasts/new)과 수정(/broadcasts/:broadcastId/edit) 공용 화면
function FormBody() {
  const { me } = useRequireLogin()
  const param = useParams().broadcastId
  const id = param === undefined ? undefined : Number(param)
  const broadcast = useBroadcast(Number.isInteger(id) ? id : undefined)
  if (!me) return null

  const heading = id === undefined ? '새 방송 등록' : '방송 정보 수정'
  const wrap = (body: React.ReactNode) => (
    <div className='mx-auto max-w-xl px-4 py-5'>
      <h1 className='mb-4 text-2xl font-black sm:text-[26px]'>{heading}</h1>
      {body}
    </div>
  )

  if (id === undefined) return <BroadcastForm heading={heading} owner={me} />
  if (broadcast.isError || !Number.isInteger(id)) return wrap(<ErrorScreen {...errorKindOf(Number.isInteger(id) ? broadcast.error : undefined)} />)
  if (!broadcast.data) return wrap(<Skeleton className='h-96 w-full' />)
  // 서버도 PATCH를 403으로 막는다. 여기서는 빈 폼 대신 안내를 보여 준다(스튜디오와 같은 문구).
  if (broadcast.data.owner.id !== me.id)
    return wrap(
      <EmptyState emote='hiding' title='내 방송이 아니에요' description='내가 등록한 방송만 수정할 수 있어요.' action={<Button to='/manage'>내 방송 관리</Button>} />,
    )
  return <BroadcastForm key={broadcast.data.id} heading={heading} owner={me} existing={broadcast.data} />
}

// 방송 등록·수정도 스튜디오 전용 상단바 아래에 그린다(시안: 사이드바 없음)
export default function BroadcastFormPage() {
  return (
    <>
      <StudioTopBar />
      <FormBody />
    </>
  )
}
