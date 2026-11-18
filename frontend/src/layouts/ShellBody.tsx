import type { ReactNode } from 'react'

// 본문 + 오른쪽 패널(채팅·방송 정보 수정 등). lg 미만에서는 패널이 본문 아래로 쌓인다(DESIGN §7).
export default function ShellBody({ aside, children }: { aside?: ReactNode; children: ReactNode }) {
  return (
    <div className='flex flex-col gap-4 lg:flex-row'>
      <div className='min-w-0 grow'>{children}</div>
      {aside && <aside className='w-full shrink-0 lg:w-80 xl:w-[360px]'>{aside}</aside>}
    </div>
  )
}
