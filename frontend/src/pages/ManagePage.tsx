import { Navigate } from 'react-router'
import { useRequireLogin } from '../features/auth/hooks/useAuth.ts'

// 내 방송 관리는 스트리머 페이지의 관리 탭으로 합쳐졌다. 옛 주소(/manage)는 거기로 보낸다.
export default function ManagePage() {
  const { me } = useRequireLogin()
  return me ? <Navigate to={`/users/${me.id}`} replace /> : null
}
