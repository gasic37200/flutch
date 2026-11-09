import { Link } from 'react-router'

// 404·라우터 오류 공통 자리. 마스코트·request_id 표시는 T05.
export default function ErrorPage() {
  return (
    <main>
      <h1>페이지를 찾을 수 없습니다</h1>
      <Link to='/'>홈으로</Link>
    </main>
  )
}
